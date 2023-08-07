use axum::{
    routing::get,
    Router,
    Json,
};
extern crate serde;

#[macro_use]
extern crate serde_derive;

pub mod metadata;
pub mod store;

use axum::extract::Path;

#[derive(Serialize, Deserialize)]
enum StatusCode {
    #[serde(rename = "OK")]
    Ok,
    #[serde(rename = "ERROR")]
    Error,
}

#[derive(Serialize, Deserialize)]
struct SimpleStatus {
    status: StatusCode,
    message: String,
}

async fn evaluate_query(Path(query): Path<String>) -> impl axum::response::IntoResponse {
    format!("Hello, {}!", query)
}

async fn submit_query(Path(query): Path<String>) -> Json<SimpleStatus> {
    Json(SimpleStatus{status:StatusCode::Ok, message:format!("Hello, {}!", query)})
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
    .route("/", get(|| async { "Hello, World!" }))
    .route("/liquer/q/*query", get(evaluate_query))
    .route("/liquer/submit/*query", get(submit_query));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}