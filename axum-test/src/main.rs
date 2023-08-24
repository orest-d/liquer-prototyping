use std::sync::{Arc, RwLock};

use axum::http::header;
use axum::{routing::get, Json, Router};
extern crate serde;

#[macro_use]
extern crate serde_derive;

pub mod error;
pub mod metadata;
pub mod parse;
pub mod query;
pub mod store;

use axum::extract::{Path, State};
use store::{FileStore, Key, Store};

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceStatus<T> {
    status: StatusCode,
    message: String,
    result: T,
}

async fn evaluate_query(Path(query): Path<String>) -> impl axum::response::IntoResponse {
    format!("Hello, {}!", query)
}

async fn submit_query(Path(query): Path<String>) -> Json<SimpleStatus> {
    Json(SimpleStatus {
        status: StatusCode::Ok,
        message: format!("Hello, {}!", query),
    })
}

/// Get data from store. Equivalent to Store.get_bytes.
/// Content type (MIME) is obtained from the metadata.
async fn store_get<S: Store>(
    State(store): State<Arc<RwLock<S>>>,
    Path(query): Path<String>,
) -> impl axum::response::IntoResponse {
    let st = store.read();
    if let Ok(store) = st {
        let data = store.get(&Key::new(query));
        if let Ok((data, metadata)) = data {
            return (
                axum::http::StatusCode::OK,
                [(header::CONTENT_TYPE, metadata.get_mimetype())],
                data,
            );
        } else {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/plain".to_owned())],
                format!("Error reading store: {}", data.err().unwrap()).into(),
            );
        }
    } else {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain".to_owned())],
            format!("Error accessing store: {}", st.err().unwrap()).into(),
        );
    }
}

/// Shortcut to the 'web' directory in the store.
/// Similar to /store/data/web, except the index.html is automatically added if query is a directory.
///The 'web' directory hosts web applications and visualization tools, e.g. liquer-pcv or liquer-gui.
async fn web_store_get<S: Store>(
    State(store): State<Arc<RwLock<S>>>,
    Path(query): Path<String>,
) -> impl axum::response::IntoResponse {
    let st = store.read();

    if let Ok(store) = st {
        let query = if query.ends_with("/") {
            format!("{}index.html", query)
        } else {
            query
        };
        let key = Key::new(&query);
        let key = if store.is_dir(&key) {
            Key::new(format!("{}/index.html", &query))
        } else {
            key
        };

        let data = store.get(&key);
        if let Ok((data, metadata)) = data {
            return (
                axum::http::StatusCode::OK,
                [(header::CONTENT_TYPE, metadata.get_mimetype())],
                data,
            );
        } else {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/plain".to_owned())],
                format!("Error reading store: {}", data.err().unwrap()).into(),
            );
        }
    } else {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain".to_owned())],
            format!("Error accessing store: {}", st.err().unwrap()).into(),
        );
    }
}

/// Set data in store. Equivalent to Store.store.
/// Unlike store method, which stores both data and metadata in one call,
/// the api/store/data POST only stores the data. The metadata needs to be set in a separate POST of api/store/metadata
/// either before or after the api/store/data POST.
async fn store_set<S: Store>(
    State(store): State<Arc<RwLock<S>>>,
    Path(query): Path<String>,
) -> impl axum::response::IntoResponse {
    let st = store.read();

    if let Ok(store) = st {
        let key = Key::new(&query);
        let metadata = store.get_metadata(&key);

        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain".to_owned())],
            format!("Store set not defined yet"),
        );
    } else {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain".to_owned())],
            format!("Store set not defined yet"),
        );
    }
}


/// Upload data to store - similar to /api/store/data, but using upload. Equivalent to Store.store.
/// Unlike store method, which stores both data and metadata in one call,
/// the api/store/data POST only stores the data. The metadata needs to be set in a separate POST of api/store/metadata
/// either before or after the api/store/data POST.
async fn store_upload<S: Store>(
    State(store): State<Arc<RwLock<S>>>,
    Path(query): Path<String>,
) -> impl axum::response::IntoResponse {
    let st = store.read();

    if let Ok(store) = st {
        let key = Key::new(&query);
        let metadata = store.get_metadata(&key);

        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain".to_owned())],
            format!("Store upload not defined yet"),
        );
    } else {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain".to_owned())],
            format!("Store upload not defined yet"),
        );
    }
}

///  Called on GET /api/store/upload/<path:query>
///  Returns a simple html interface that allows to upload content to the store.
async fn store_upload_get(
    Path(query): Path<String>,
) -> impl axum::response::IntoResponse {
    let key = Key::new(&query);
    (
        axum::http::StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html".to_owned())],
        format!("<!doctype html>
        <title>Upload File</title>
        <h1>Upload to {}</h1>
        <form method=\"post\" enctype=\"multipart/form-data\">
        <input type=\"file\" name=\"file\"/>
        <input type=\"submit\" value=\"Upload\"/>
        </form>", key
    )
    )
}


/* 
@app.route("/api/store/metadata/<path:query>", methods=["GET"])
def store_get_metadata(query):
    store = get_store()
    metadata = store.get_metadata(query)
    return jsonify(metadata)


@app.route("/api/store/metadata/<path:query>", methods=["POST"])
def store_set_metadata(query):
    store = get_store()
    try:
        metadata = request.get_json(force=True)
        store.store_metadata(query, metadata)

        return jsonify(dict(query=query, message="Metadata stored", status="OK"))
    except:
        response = jsonify(
            dict(query=query, message=traceback.format_exc(), status="ERROR")
        )
        response.status = "404"
        return response


@app.route("/api/stored_metadata/<path:query>", methods=["GET"])
def get_stored_metadata(query):
    """Get metadata stored in a store or cache"""
    import liquer.tools

    metadata = liquer.tools.get_stored_metadata(query)
    return jsonify(metadata)


@app.route("/api/store/remove/<path:query>")
def store_remove(query):
    store = get_store()
    try:
        store.remove(query)
        return jsonify(dict(query=query, message=f"Removed {query}", status="OK"))
    except:
        return jsonify(
            dict(query=query, message=traceback.format_exc(), status="ERROR")
        )


@app.route("/api/store/removedir/<path:query>")
def store_removedir(query):
    store = get_store()
    try:
        store.removedir(query)
        return jsonify(
            dict(query=query, message=f"Removed directory {query}", status="OK")
        )
    except:
        return jsonify(
            dict(query=query, message=traceback.format_exc(), status="ERROR")
        )


@app.route("/api/store/contains/<path:query>")
def store_contains(query):
    store = get_store()
    try:
        contains = store.contains(query)
        return jsonify(
            dict(
                query=query, message=f"Contains {query}", contains=contains, status="OK"
            )
        )
    except:
        return jsonify(
            dict(query=query, message=traceback.format_exc(), status="ERROR")
        )


@app.route("/api/store/is_dir/<path:query>")
def store_is_dir(query):
    store = get_store()
    try:
        is_dir = store.is_dir(query)
        return jsonify(
            dict(
                query=query, message=f"Is directory {query}", is_dir=is_dir, status="OK"
            )
        )
    except:
        return jsonify(
            dict(query=query, message=traceback.format_exc(), status="ERROR")
        )


@app.route("/api/store/keys")
def store_keys():
    store = get_store()
    try:
        keys = store.keys()
        return jsonify(
            dict(query=None, message=f"Keys obtained", keys=keys, status="OK")
        )
    except:
        return jsonify(dict(query=None, message=traceback.format_exc(), status="ERROR"))


@app.route("/api/store/listdir/<path:query>")
def store_listdir(query):
    store = get_store()
    try:
        listdir = store.listdir(query)
        return jsonify(
            dict(query=query, message=f"Keys obtained", listdir=listdir, status="OK")
        )
    except:
        return jsonify(
            dict(query=query, message=traceback.format_exc(), status="ERROR")
        )


@app.route("/api/store/makedir/<path:query>")
def store_makedir(query):
    store = get_store()
    try:
        store.makedir(query)
        return jsonify(dict(query=query, message=f"Makedir succeeded", status="OK"))
    except:
        return jsonify(
            dict(query=query, message=traceback.format_exc(), status="ERROR")
        )
*/

#[tokio::main]
async fn main() {
    // build our application with a single route

    let shared_state = Arc::new(RwLock::new(FileStore::new(".", "")));

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/liquer/q/*query", get(evaluate_query))
        .route("/liquer/submit/*query", get(submit_query))
        .route("/liquer/store/data/*query", get(store_get))
        .route("/liquer/web/*query", get(web_store_get))
        .route("/liquer/store/upload/*query", get(store_upload_get))
        .with_state(shared_state);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
