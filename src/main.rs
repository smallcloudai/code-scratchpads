// use reqwest::header::AUTHORIZATION;
// use ropey::Rope;
use std::convert::Infallible;
use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
// use serde_json::Error as SerdeJsonError;
// use std::collections::HashMap;

// use std::sync::Arc;

// use tokenizers::Tokenizer;

// use tokio::io::AsyncWriteExt;
// use tokio::sync::RwLock;
// use async_trait::async_trait;

use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use hyper::server::conn::{AddrStream};
use hyper::service::{make_service_fn, service_fn};

use route_recognizer::{Match, Params, Router};

// use tower::ServiceBuilder;

use tracing::{error, info};

// https://blog.logrocket.com/a-minimal-web-service-in-rust-using-hyper/


#[derive(Debug)]
struct MyContext {
    router: Router<String>,
}

#[derive(Debug, Deserialize)]
struct MyRequest {
    model: String,
}


async fn handle_v1_code_completion(
    body_bytes: hyper::body::Bytes
) -> Result<Response<Body>, hyper::Error> {
    let my_request_result = serde_json::from_slice::<MyRequest>(&body_bytes);
    let my_request: MyRequest = match my_request_result {
        Ok(x) => x,
        Err(e) => {
            error!("Error deserializing request body: {}", e);
            return Ok(Response::builder()
               .status(hyper::StatusCode::BAD_REQUEST)
              .body(format!("could not parse JSON: {}", e).into())
              .unwrap()
              .into());
        }
    };
    let txt = format!("hurray a call! model was: {}\n",
        my_request.model,
        );
    info!("handle_v1_code_completion returning: {}", txt);
    let response = Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(txt))
        .unwrap();
    Ok(response)
}

async fn handle_request(
    remote_addr: SocketAddr,
    path: String,
    method: Method,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    if method!= Method::POST {
        return Ok(Response::builder()
           .status(StatusCode::METHOD_NOT_ALLOWED)
           .body(Body::from("Method not allowed"))
           .unwrap());
    }
    let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
    if path.starts_with("/v1/code-completion") {
        return handle_v1_code_completion(body_bytes).await;
    }
    let txt = format!("not found path {}, your ip is {}\n",
        path,
        remote_addr);
    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "application/json")
        .body(Body::from(txt))
        .unwrap();
    Ok(response)
}


#[tokio::main]
async fn main() {
    let _builder1 = tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .with_target(true)
        .with_line_number(true)
        .compact()
        .init();
    let make_svc = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let path = req.uri().path().to_string();
                let method = req.method().clone();
                handle_request(remote_addr, path, method, req)
            }))
        }
    });
    let addr = ([127, 0, 0, 1], 8008).into();
    let server = Server::bind(&addr).serve(make_svc);
    println!("Server listening on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    // let http_client = reqwest::Client::new();
    // let (service, socket) = LspService::build(|client| Backend {
    //     cache_dir,
    //     client,
    //     document_map: Arc::new(RwLock::new(HashMap::new())),
    //     http_client,
    //     workspace_folders: Arc::new(RwLock::new(None)),
    //     tokenizer_map: Arc::new(RwLock::new(HashMap::new())),
    // })
    // .custom_method("llm-ls/getCompletions", Backend::get_completions)
    // .finish();
    // Server::new(stdin, stdout, socket).serve(service).await;
}
