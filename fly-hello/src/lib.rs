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
use std::io::Write;
use url::Url;
use wasi::http::types::*;
use wasi::logging::logging::*;
use wasmcloud::bus::host::*;

#[derive(RustEmbed)]
#[folder = "dist"]
struct Assets;

struct HttpServer;

// This struct implementation is from
// https://github.com/wasmCloud/wasmCloud/blob/ef3955a754ab59d2597dd1ef1801ac667eaf19a5/crates/actor/src/wrappers/io.rs#L45-L89
// Copied here for convenience so we don't have to depend on the wasmcloud crate that implements
// it.
pub struct OutputStreamWriter<'a> {
    stream: &'a mut crate::wasi::io::streams::OutputStream,
}

impl<'a> From<&'a mut crate::wasi::io::streams::OutputStream> for OutputStreamWriter<'a> {
    fn from(stream: &'a mut crate::wasi::io::streams::OutputStream) -> Self {
        Self { stream }
    }
}

impl std::io::Write for OutputStreamWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        use crate::wasi::io::streams::StreamError;
        use std::io;

        let n = match self.stream.check_write().map(std::num::NonZeroU64::new) {
            Ok(Some(n)) => n,
            Ok(None) | Err(StreamError::Closed) => return Ok(0),
            Err(StreamError::LastOperationFailed(e)) => {
                return Err(io::Error::new(io::ErrorKind::Other, e.to_debug_string()))
            }
        };
        let n = n
            .get()
            .try_into()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let n = buf.len().min(n);
        self.stream.write(&buf[..n]).map_err(|e| match e {
            StreamError::Closed => io::ErrorKind::UnexpectedEof.into(),
            StreamError::LastOperationFailed(e) => {
                io::Error::new(io::ErrorKind::Other, e.to_debug_string())
            }
        })?;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream
            .blocking_flush()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl Guest for HttpServer {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        let response = OutgoingResponse::new(Fields::new());
        let path = request.path_with_query().unwrap();
        log(Level::Info, "fly-hello", path.as_str());
        let headers = request.headers();
        let host_key = "Host".to_string();

        // this panics when curling to localhost
        //request.authority().unwrap();
        // Instead you need to do this nonsense until wRPC is finished.
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

        // Apparently wasmtime is really sensitive to holding resources for too long, so this
        // ensures that the writer is fully dropped by the time we want to finish the body stream.
        {
            let mut writer = response_body.write().unwrap();
            let mut w = OutputStreamWriter::from(&mut writer);
            w.write_all(data.as_bytes()).expect("failed to write");
            w.flush().expect("failed to flush");
        }

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
