#![allow(clippy::missing_safety_doc)]
wit_bindgen::generate!();

use axum::{
    extract::Path,
    http::{HeaderMap, HeaderName, HeaderValue},
    response::Response,
    routing::get,
    Router,
};
use cosmonic_labs::cloud_metadata::service;
use exports::wasi::http::incoming_handler::Guest;
use handlebars::Handlebars;
use http::Request;
use rust_embed::RustEmbed;
use serde::Serialize;
use serde_json::json;
use std::{collections::BTreeMap, io::Write};
use tower_service::Service;
use wasi::http::types::*;
use wasi::logging::logging::*;
use wrpc::keyvalue::{atomics, store};

mod helpers;
use helpers::*;

#[derive(RustEmbed)]
#[folder = "ui/dist"]
struct Assets;

#[derive(Serialize)]
struct UAStats {
    os: BTreeMap<String, u64>,
    browsers: BTreeMap<String, u64>,
    visits: u64,
}

type RegionTotals = BTreeMap<String, UAStats>;

const COMPONENT_NAME: &str = "cloud-hello";

struct HttpServer;

impl HttpServer {
    fn router() -> Router {
        Router::new()
            .route("/", get(HttpServer::index))
            .route("/assets/:path", get(HttpServer::asset))
            .route("/health", get(HttpServer::health))
    }

    async fn health() -> Response {
        Response::builder()
            .status(200)
            .body(axum::body::Body::empty())
            .unwrap()
    }

    async fn index(headers: HeaderMap) -> Response {
        let metadata = match service::get() {
            Ok(m) => m,
            Err(e) => {
                log(
                    Level::Error,
                    COMPONENT_NAME,
                    format!("Error getting metadata: {}", e).as_str(),
                );
                return Response::builder()
                    .status(500)
                    .body(axum::body::Body::empty())
                    .unwrap();
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

        if let Some(user_agent) = headers.get("User-Agent") {
            let user_agent_header = user_agent.as_bytes();
            let parser = woothee::parser::Parser::new();
            let result = parser.parse(std::str::from_utf8(user_agent_header).unwrap());
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
                Level::Debug,
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

        Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(axum::body::Body::from(data))
            .unwrap()
    }

    async fn asset(Path(path): Path<String>) -> Response {
        if let Some(asset) = Assets::get(format!("assets/{path}").as_str()) {
            let mut response = Response::builder()
                .status(200)
                .body(axum::body::Body::from(asset.data.to_vec()))
                .unwrap();
            if path.contains(".js") {
                response.headers_mut().insert(
                    "Content-Type",
                    HeaderValue::from_static("application/javascript"),
                );
            }
            return response;
        }

        Response::builder()
            .status(404)
            .body(axum::body::Body::new("".to_string()))
            .unwrap()
    }

    async fn to_wasi_response(resp: Response) -> OutgoingResponse {
        let fields = Fields::new();
        if !resp.headers().is_empty() {
            for (k, v) in resp.headers() {
                let value = [v.to_str().unwrap().to_string().into_bytes()];
                fields.set(&k.as_str().to_string(), &value).unwrap();
            }
        }
        let status = resp.status().as_u16();

        let body = resp.into_body();
        let b = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let b2 = b.to_vec();

        let response = OutgoingResponse::new(fields);
        response.set_status_code(status).unwrap();
        let response_body = response.body().unwrap();
        // Apparently wasmtime is really sensitive to holding resources for too long, so this
        // ensures that the writer is fully dropped by the time we want to finish the body stream.
        {
            let mut writer = response_body.write().unwrap();
            let mut w = OutputStreamWriter::from(&mut writer);
            w.write_all(&b2).expect("failed to write");
            w.flush().expect("failed to flush");
        }

        OutgoingBody::finish(response_body, None).expect("failed to finish response body");
        response
    }
}

impl Guest for HttpServer {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
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

        let mut router = HttpServer::router();
        let path = request.path_with_query().unwrap();
        log(Level::Info, "http", &format!("path: {}", path));
        let method = request.method().to_string();

        let mut req = Request::builder().method(method.as_str()).uri(path);
        let m = req.headers_mut().unwrap();
        let entries = headers.entries().clone();

        let entries: Vec<(String, Vec<u8>)> =
            entries.into_iter().map(|(k, v)| (k.clone(), v)).collect();

        for (k, v) in entries {
            let hn = HeaderName::from_bytes(k.as_bytes()).unwrap();
            let hv = HeaderValue::from_bytes(v.as_slice()).unwrap();
            m.insert(hn, hv);
        }

        let r = req.body(axum::body::Body::empty()).unwrap();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = rt.block_on(async { router.call(r).await.unwrap() });
        let response = rt.block_on(async { HttpServer::to_wasi_response(result).await });
        ResponseOutparam::set(response_out, Ok(response));
    }
}

// NOTE: since wit-bindgen creates these types in our namespace,
// we can hang custom implementations off of them
impl ToString for Method {
    fn to_string(&self) -> String {
        match self {
            Method::Get => "GET".into(),
            Method::Post => "POST".into(),
            Method::Patch => "PATCH".into(),
            Method::Put => "PUT".into(),
            Method::Delete => "DELETE".into(),
            Method::Options => "OPTIONS".into(),
            Method::Head => "HEAD".into(),
            Method::Connect => "CONNECT".into(),
            Method::Trace => "TRACE".into(),
            Method::Other(m) => m.into(),
        }
    }
}
export!(HttpServer);
