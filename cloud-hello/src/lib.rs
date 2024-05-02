wit_bindgen::generate!();

use cosmonic_labs::cloud_metadata::service;
use exports::wasi::http::incoming_handler::Guest;
use handlebars::Handlebars;
use rust_embed::RustEmbed;
use serde::Serialize;
use serde_json::json;
use std::{collections::BTreeMap, io::Write};
use url::Url;
use wasi::http::types::*;
use wasi::logging::logging::*;
use wrpc::keyvalue::{atomics, batch, store};

#[derive(RustEmbed)]
#[folder = "ui/dist"]
struct Assets;

struct HttpServer;

#[derive(Serialize)]
struct UAStats {
    os: BTreeMap<String, u64>,
    browsers: BTreeMap<String, u64>,
    visits: u64,
}

type RegionTotals = BTreeMap<String, UAStats>;

const COMPONENT_NAME: &str = "cloud-hello";

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
        log(Level::Info, COMPONENT_NAME, "handling request");
        let path = request.path_with_query().unwrap();
        log(Level::Info, COMPONENT_NAME, path.as_str());
        let headers = request.headers();

        log(
            Level::Info,
            COMPONENT_NAME,
            format!("path {}", path).as_str(),
        );
        log(
            Level::Info,
            COMPONENT_NAME,
            format!("headers {:?}", headers).as_str(),
        );

        // this panics when curling to localhost
        //request.authority().unwrap();
        // Instead you need to do this nonsense until wRPC is finished.
        //let host_header = headers.get(&host_key)[0].clone();
        //let host = std::str::from_utf8(&host_header).unwrap();
        // TODO replace with axum's router
        let url = match Url::parse(format!("whatever:{}", path.as_str()).as_str()) {
            Ok(u) => u,
            Err(e) => {
                log(
                    Level::Info,
                    COMPONENT_NAME,
                    format!("error: {}", e).as_str(),
                );
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
                    COMPONENT_NAME,
                    format!("Error getting metadata: {}", e).as_str(),
                );
                let response = error(format!("Error getting metadata: {}", e));
                ResponseOutparam::set(response_out, Ok(response));
                return;
            }
        };

        let region = metadata.region;
        let code = region.code.clone().unwrap_or_default();

        // wasi:keyvalue code
        // The redis store doesn't support a bucket name
        //let bucket = match store::open("") {
        //    Ok(b) => b,
        //    Err(e) => {
        //        log(
        //            Level::Error,
        //            "fly-hello",
        //            format!("Error opening store: {}", e).as_str(),
        //        );
        //        let response = error(format!("Error opening store: {}", e));
        //        ResponseOutparam::set(response_out, Ok(response));
        //        return;
        //    }
        //};

        if let Err(e) = atomics::increment("", code.as_str(), 1) {
            log(
                Level::Error,
                COMPONENT_NAME,
                format!("Error incrementing key: {}", e).as_str(),
            );
        }

        let user_agent_key = "User-Agent".to_string();
        let user_agent_header = headers.get(&user_agent_key);
        if !user_agent_header.is_empty() {
            let user_agent_header = user_agent_header[0].clone();
            let parser = woothee::parser::Parser::new();
            let result = parser.parse(std::str::from_utf8(&user_agent_header).unwrap());
            if let Some(res) = result {
                if let Err(e) =
                    atomics::increment("", format!("{}:browser:{}", code, res.name).as_str(), 1)
                {
                    log(
                        Level::Error,
                        COMPONENT_NAME,
                        format!("Error incrementing key: {}", e).as_str(),
                    );
                };
                if let Err(e) =
                    atomics::increment("", format!("{}:os:{}", code, res.os).as_str(), 1)
                {
                    log(
                        Level::Error,
                        COMPONENT_NAME,
                        format!("Error incrementing key: {}", e).as_str(),
                    );
                };
            }
        };

        let mut keys = Vec::new();
        let mut cursor = None;
        loop {
            match store::list_keys("", cursor) {
                Ok(k) => {
                    keys.extend(k.keys);
                    if k.cursor.is_none() {
                        break;
                    }
                    cursor = k.cursor;
                }
                Err(e) => {
                    log(
                        Level::Error,
                        COMPONENT_NAME,
                        format!("Error listing keys: {}", e).as_str(),
                    );
                    break;
                }
            };
        }

        let mut totals = RegionTotals::new();

        // wasi:keyvalue code
        //let mut region_counts = BTreeMap::new();
        //for key in keys {
        //    log(Level::Info, "fly-hello", format!("key: {}", key).as_str());
        //    if let Ok(Some(v)) = bucket.get(key.as_str()) {
        //        region_counts.insert(key, String::from_utf8_lossy(&v).to_string());
        //    };
        //}
        let mut data = BTreeMap::new();
        for key in keys {
            log(
                Level::Info,
                COMPONENT_NAME,
                format!("key: {}", key).as_str(),
            );
            if let Ok(Some(v)) = store::get("", key.as_str()) {
                data.insert(key, String::from_utf8_lossy(&v).to_string());
            };
        }

        data.iter()
            .filter(|(k, _)| !k.contains(':'))
            .for_each(|(k, v)| {
                totals
                    .entry(k.clone())
                    .or_insert(UAStats {
                        os: BTreeMap::new(),
                        browsers: BTreeMap::new(),
                        visits: 0,
                    })
                    .visits += v.parse::<u64>().unwrap();
            });

        data.iter()
            .filter(|(k, _)| k.contains(":os:"))
            .for_each(|(k, v)| {
                let parts = k.split(':').collect::<Vec<&str>>();
                let region = parts[0].to_string();
                let os = parts[2].to_string();
                let count = v.parse::<u64>().unwrap();
                totals
                    .entry(region.clone())
                    .or_insert(UAStats {
                        os: BTreeMap::new(),
                        browsers: BTreeMap::new(),
                        visits: 0,
                    })
                    .os
                    .entry(os)
                    .and_modify(|e| *e += count)
                    .or_insert(count);
            });

        data.iter()
            .filter(|(k, _)| k.contains(":browser:"))
            .for_each(|(k, v)| {
                let parts = k.split(':').collect::<Vec<&str>>();
                let region = parts[0].to_string();
                let browser = parts[2].to_string();
                let count = v.parse::<u64>().unwrap();
                totals
                    .entry(region.clone())
                    .or_insert(UAStats {
                        os: BTreeMap::new(),
                        browsers: BTreeMap::new(),
                        visits: 0,
                    })
                    .browsers
                    .entry(browser)
                    .and_modify(|e| *e += count)
                    .or_insert(count);
            });

        let region_json = serde_json::to_string(&totals).unwrap();

        let template = Assets::get("index.html").unwrap();
        let reg = Handlebars::new();
        let data = reg
            .render_template(
                std::str::from_utf8(&template.data).unwrap(),
                &json!({"code": code, "location": region.name, "region_data": totals, "region_json": region_json}),
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
    log(Level::Info, COMPONENT_NAME, url.path());
    let path = url.path().strip_prefix('/').unwrap();
    if let Some(asset) = Assets::get(path) {
        if path.contains(".js") {
            let value = ["application/javascript".to_string().into_bytes()];
            if let Err(e) = fields.set(&"Content-Type".to_string(), &value) {
                log(
                    Level::Error,
                    COMPONENT_NAME,
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
