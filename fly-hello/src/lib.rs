wit_bindgen::generate!({
    world: "hello",
    exports: {
        "wasi:http/incoming-handler": HttpServer,
    },
});

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
        let mut region: String = "".to_string();
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
                &json!({"region": region}),
            )
            .unwrap();

        response.set_status_code(200).unwrap();
        let response_body = response.body().unwrap();
        response_body
            .write()
            .unwrap()
            .blocking_write_and_flush(data.as_bytes())
            .unwrap();
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
