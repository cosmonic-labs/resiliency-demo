wit_bindgen::generate!();

use cosmonic_labs::cloud_metadata::service;
use exports::wasi::http::incoming_handler::Guest;
//use fly_metadata::*;
use handlebars::Handlebars;
use rust_embed::RustEmbed;
use serde_json::json;
use std::{collections::BTreeMap, io::Write};
use url::Url;
use wasi::http::types::*;
use wasi::keyvalue::{atomics, batch, store};
use wasi::logging::logging::*;

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
        log(Level::Info, "fly-hello", "handling request");
        let path = request.path_with_query().unwrap();
        log(Level::Info, "fly-hello", path.as_str());
        let headers = request.headers();
        let host_key = "Host".to_string();

        log(Level::Info, "fly-hello", format!("path {}", path).as_str());
        log(
            Level::Info,
            "fly-hello",
            format!("headers {:?}", headers).as_str(),
        );

        // this panics when curling to localhost
        //request.authority().unwrap();
        // Instead you need to do this nonsense until wRPC is finished.
        //let host_header = headers.get(&host_key)[0].clone();
        //let host = std::str::from_utf8(&host_header).unwrap();
        let url = match Url::parse(format!("whatever:{}", path.as_str()).as_str()) {
            Ok(u) => u,
            Err(e) => {
                log(Level::Info, "fly-hello", format!("error: {}", e).as_str());
                let response = error(format!("Error parsing URL: {}", e));
                ResponseOutparam::set(response_out, Ok(response));
                return;
            }
        };

        if url.path() != "/" && url.path() != "/index.html" {
            let resp = handle_asset(url);
            ResponseOutparam::set(response_out, Ok(resp));
            return;
        }

        let metadata = match service::get() {
            Ok(m) => m,
            Err(e) => {
                log(
                    Level::Error,
                    "fly-hello",
                    format!("Error getting metadata: {}", e).as_str(),
                );
                let response = error(format!("Error getting metadata: {}", e));
                ResponseOutparam::set(response_out, Ok(response));
                return;
            }
        };

        let region = metadata.region;
        let code = region.code.clone().unwrap_or_default();

        // The redis store doesn't support a bucket name
        let bucket = match store::open("") {
            Ok(b) => b,
            Err(e) => {
                log(
                    Level::Error,
                    "fly-hello",
                    format!("Error opening store: {}", e).as_str(),
                );
                let response = error(format!("Error opening store: {}", e));
                ResponseOutparam::set(response_out, Ok(response));
                return;
            }
        };

        // TODO do something with the user agent?
        //let user_agent_key = "User-Agent".to_string();
        //let user_agent_header = headers.get(&user_agent_key)[0].clone();
        //if let Err(e) = bucket.set("foo", &user_agent_header) {
        //    log(
        //        Level::Error,
        //        "fly-hello",
        //        format!("Error setting key: {}", e).as_str(),
        //    );
        //    let response = error(format!("Error setting key: {}", e));
        //    ResponseOutparam::set(response_out, Ok(response));
        //    return;
        //};
        if let Err(e) = atomics::increment(&bucket, code.as_str(), 1) {
            log(
                Level::Error,
                "fly-hello",
                format!("Error incrementing key: {}", e).as_str(),
            );
            let response = error(format!("Error incrementing key: {}", e));
            ResponseOutparam::set(response_out, Ok(response));
            return;
        }

        let mut keys = Vec::new();
        loop {
            match bucket.list_keys(None) {
                Ok(k) => {
                    keys.extend(k.keys);
                    if k.cursor.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    log(
                        Level::Error,
                        "fly-hello",
                        format!("Error listing keys: {}", e).as_str(),
                    );
                    break;
                }
            };
        }

        let mut region_counts = BTreeMap::new();
        for key in keys {
            log(Level::Info, "fly-hello", format!("key: {}", key).as_str());
            if let Ok(Some(v)) = bucket.get(key.as_str()) {
                region_counts.insert(key, String::from_utf8_lossy(&v).to_string());
            };
        }

        //let values = match batch::get_many(&bucket, &keys) {
        //    Ok(v) => v.iter().filter_map(|v| v.as_ref()).cloned().fold(
        //        BTreeMap::new(),
        //        |mut acc, (k, v)| {
        //            acc.insert(k, String::from_utf8_lossy(&v).to_string());
        //            acc
        //        },
        //    ),
        //    Err(e) => {
        //        log(
        //            Level::Error,
        //            "fly-hello",
        //            format!("Error getting values: {}", e).as_str(),
        //        );
        //        BTreeMap::new()
        //    }
        //};

        let template = Assets::get("index.html").unwrap();
        let reg = Handlebars::new();
        let data = reg
            .render_template(
                std::str::from_utf8(&template.data).unwrap(),
                &json!({"code": code, "location": region.name, "region_data": region_counts}),
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

fn error(message: String) -> OutgoingResponse {
    let response = OutgoingResponse::new(Fields::new());
    response.set_status_code(500).unwrap();
    let response_body = response.body().unwrap();
    response_body.write().unwrap();
    response_body
        .write()
        .unwrap()
        .blocking_write_and_flush(format!("Error: {}", message).as_bytes())
        .unwrap();
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
    let fields = Fields::new();
    log(Level::Info, "fly-hello", url.path());
    let path = url.path().strip_prefix('/').unwrap();
    if let Some(asset) = Assets::get(path) {
        if path.contains(".js") {
            let value = ["application/javascript".to_string().into_bytes()];
            if let Err(e) = fields.set(&"Content-Type".to_string(), &value) {
                log(
                    Level::Error,
                    "fly-hello",
                    format!("Error setting header: {}", e).as_str(),
                )
            }
        }

        let response = OutgoingResponse::new(fields);
        response.set_status_code(200).unwrap();
        let response_body = response.body().unwrap();
        response_body.write().unwrap().write(&asset.data).unwrap();
        OutgoingBody::finish(response_body, None).expect("failed to finish response body");
        return response;
    }
    not_found()
}
export!(HttpServer);
