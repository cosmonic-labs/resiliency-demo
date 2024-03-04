wit_bindgen::generate!({
    world: "hello",
    exports: {
        "wasi:http/incoming-handler": HttpServer,
    },
});

use std::io::{BufRead, BufReader};

use crate::wasmcloud::bus::lattice;
use exports::wasi::http::incoming_handler::Guest;
use fly_metadata::*;
use handlebars::Handlebars;
use rust_embed::RustEmbed;
use serde_json::json;
use url::Url;
use wasi::http::types::*;
use wasi::logging::logging::*;
use wasmcloud::bus::host::*;

const BUF_CHUNK_READ: usize = 4096;

#[derive(RustEmbed)]
#[folder = "dist"]
struct Assets;

struct HttpServer;

impl Guest for HttpServer {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        let response = OutgoingResponse::new(Fields::new());
        let path = request.path_with_query().unwrap();
        log(Level::Info, "fly-hello", path.as_str());
        let headers = request.headers();
        let host_key = "Host".to_string();

        // this panics when curling to localhost
        //request.authority().unwrap();
        // Instead you need to do this nonsense
        let host_header = headers.get(&host_key)[0].clone();
        let host = std::str::from_utf8(&host_header).unwrap();
        let url = match Url::parse(format!("http://{}{}", host, path.as_str()).as_str()) {
            Ok(u) => u,
            Err(e) => {
                log(Level::Info, "fly-hello", format!("error: {}", e).as_str());
                let response = OutgoingResponse::new(Fields::new());
                response.set_status_code(500).unwrap();
                let response_body = response.body().unwrap();
                OutgoingBody::finish(response_body, None).expect("failed to finish response body");
                ResponseOutparam::set(response_out, Ok(response));
                return;
            }
        };

        if url.path() != "/" && url.path() != "/index.html" {
            let resp = handle_asset(url);
            ResponseOutparam::set(response_out, Ok(resp));
            return;
        }

        let target = lattice::TargetEntity::Link(None);
        let resp = call_sync(Some(&target), "protochron:fly_metadata/Metadata.Get", &[]);
        let mut region: Region = Region::default();
        if let Ok(r) = resp {
            log(Level::Info, "fly-hello", "Got response from Metadata.Get");
            let get_response = match wasmbus_rpc::common::deserialize::<GetResponse>(&r) {
                Ok(get) => get,
                Err(_) => {
                    let err = error();
                    ResponseOutparam::set(response_out, Ok(err));
                    return;
                }
            };
            log(
                Level::Info,
                "fly-hello",
                &format!("Got response: {:?}", get_response),
            );

            region = get_response.region;
        } else {
            log(Level::Error, "fly-hello", "Error calling Metadata.Get");
            if let Err(e) = resp {
                let response = OutgoingResponse::new(Fields::new());
                response.set_status_code(500).unwrap();
                let response_body = response.body().unwrap();
                response_body
                    .write()
                    .unwrap()
                    .blocking_write_and_flush(format!("Error: {}", e).as_bytes())
                    .unwrap();
                OutgoingBody::finish(response_body, None).expect("failed to finish response body");
                ResponseOutparam::set(response_out, Ok(response));
                return;
            }
        }

        let template = Assets::get("index.html").unwrap();
        let reg = Handlebars::new();
        let data = reg
            .render_template(
                std::str::from_utf8(&template.data).unwrap(),
                &json!({"code": region.code, "location": region.name}),
            )
            .unwrap();

        response.set_status_code(200).unwrap();
        let response_body = response.body().unwrap();
        let writer = response_body.write().unwrap();

        // Have to read in 4096 byte chunks because that's the max size we can write to a stream at
        // a time.
        let mut reader = BufReader::with_capacity(BUF_CHUNK_READ, data.as_bytes());
        loop {
            let buffer = reader.fill_buf().unwrap();
            let buf_len = buffer.len();
            if buf_len == 0 {
                break;
            }
            let amt = writer.check_write().expect("unable to check write");
            if amt < BUF_CHUNK_READ as u64 {
                writer.blocking_flush().expect("failed to flush");
            }
            writer.write(buffer).expect("unable to write from buffer");
            reader.consume(buf_len);
        }

        writer.flush().expect("failed to flush response writer");
        OutgoingBody::finish(response_body, None).expect("failed to finish response body");
        ResponseOutparam::set(response_out, Ok(response));
    }
}

fn error() -> OutgoingResponse {
    let response = OutgoingResponse::new(Fields::new());
    response.set_status_code(500).unwrap();
    let response_body = response.body().unwrap();
    response_body.write().unwrap();
    OutgoingBody::finish(response_body, None).expect("failed to finish response body");
    response
}

fn not_found() -> OutgoingResponse {
    let response = OutgoingResponse::new(Fields::new());
    response.set_status_code(404).unwrap();
    let response_body = response.body().unwrap();
    response_body.write().unwrap();
    OutgoingBody::finish(response_body, None).expect("failed to finish response body");
    response
}

fn handle_asset(url: Url) -> OutgoingResponse {
    let response = OutgoingResponse::new(Fields::new());
    log(Level::Info, "fly-hello", url.path());
    let path = url.path().strip_prefix('/').unwrap();
    if let Some(asset) = Assets::get(path) {
        response.set_status_code(200).unwrap();
        let response_body = response.body().unwrap();
        response_body.write().unwrap().write(&asset.data).unwrap();
        OutgoingBody::finish(response_body, None).expect("failed to finish response body");
        return response;
    }
    not_found()
}
