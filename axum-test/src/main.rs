use std::sync::{RwLock, Arc};

use axum::http::header;
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

use axum::extract::{Path, State};
use store::{Store, FileStore, Key};

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

/// Get data from store. Equivalent to Store.get_bytes.
/// Content type (MIME) is obtained from the metadata.
async fn store_get<S:Store>(State(store):State<Arc<RwLock<S>>>, Path(query): Path<String>) -> impl axum::response::IntoResponse {
    let st = store.read();
    if let Ok(store) = st {
        let data = store.get(&Key::new(query));
        if let Ok((data, metadata)) = data {
            return (axum::http::StatusCode::OK,
                [(header::CONTENT_TYPE, metadata.get_mimetype())],
                data);
        }
        else{
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/plain".to_owned())],
                format!("Error reading store: {}", data.err().unwrap()).into());       
        }
    }
    else {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain".to_owned())],
            format!("Error accessing store: {}", st.err().unwrap()).into());
    }
}

/// Shortcut to the 'web' directory in the store.
/// Similar to /store/data/web, except the index.html is automatically added if query is a directory.
///The 'web' directory hosts web applications and visualization tools, e.g. liquer-pcv or liquer-gui.
async fn web_store_get<S:Store>(State(store):State<Arc<RwLock<S>>>, Path(query): Path<String>) -> impl axum::response::IntoResponse {
    let st = store.read();
    
    if let Ok(store) = st {
        let query = if query.ends_with("/") {
            format!("{}index.html",query)
        }
        else {
            query
        };
        let key=Key::new(&query);
        let key = if store.is_dir(&key) {
            Key::new(format!("{}/index.html",&query))
        }
        else {
            key
        };

        let data = store.get(&key);
        if let Ok((data, metadata)) = data {
            return (axum::http::StatusCode::OK,
                [(header::CONTENT_TYPE, metadata.get_mimetype())],
                data);
        }
        else{
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/plain".to_owned())],
                format!("Error reading store: {}", data.err().unwrap()).into());       
        }
    }
    else {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain".to_owned())],
            format!("Error accessing store: {}", st.err().unwrap()).into());
    }
}
/* 

@app.route("/api/store/data/<path:query>", methods=["POST"])
def store_set(query):
    """Set data in store. Equivalent to Store.store.
    Unlike store method, which stores both data and metadata in one call,
    the api/store/data POST only stores the data. The metadata needs to be set in a separate POST of api/store/metadata
    either before or after the api/store/data POST.
    """
    store = get_store()
    try:
        metadata = store.get_metadata(query)
    except KeyNotFoundStoreException:
        metadata = {}
        traceback.print_exc()
    try:
        data = request.get_data()
        store.store(query, data, metadata)
        return jsonify(dict(query=query, message="Data stored", status="OK"))
    except:
        response = jsonify(
            dict(query=query, message=traceback.format_exc(), status="ERROR")
        )
        response.status = "404"
        return response


@app.route("/api/store/upload/<path:query>", methods=["GET", "POST"])
def store_upload(query):
    """Upload data to store - similar to /api/store/data, but using upload. Equivalent to Store.store.
    Unlike store method, which stores both data and metadata in one call,
    the api/store/data POST only stores the data. The metadata needs to be set in a separate POST of api/store/metadata
    either before or after the api/store/data POST.
    """
    if request.method == "POST":
        if "file" not in request.files:
            response = jsonify(
                dict(
                    query=query,
                    message="Request does not contain 'file'",
                    status="ERROR",
                )
            )
            response.status = "404"
            return response
        file = request.files["file"]
        if file.filename == "":
            response = jsonify(
                dict(
                    query=query,
                    message="Request contains 'file' with an empty filename",
                    status="ERROR",
                )
            )
            response.status = "404"
            return response

        try:
            data = file.read()
        except:
            response = jsonify(
                dict(query=query, message=traceback.format_exc(), status="ERROR")
            )
            response.status = "404"
            return response
        store = get_store()
        try:
            metadata = store.get_metadata(query)
        except KeyNotFoundStoreException:
            metadata = {}
            traceback.print_exc()
        try:
            store.store(query, data, metadata)
            return jsonify(
                dict(query=query, message="Data stored", size=len(data), status="OK")
            )
        except:
            response = jsonify(
                dict(query=query, message=traceback.format_exc(), status="ERROR")
            )
            response.status = "404"
            return response

    r = make_response(
        f"""
    <!doctype html>
    <title>Upload File</title>
    <h1>Upload to {query}</h1>
    <form method="post" enctype="multipart/form-data">
      <input type="file" name="file"/>
      <input type="submit" value="Upload"/>
    </form>
    """
    )

    r.headers.set("Content-Type", "text/html")
    return r


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

    let shared_state = Arc::new(RwLock::new(FileStore::new(".","")));

    let app = Router::new()
    .route("/", get(|| async { "Hello, World!" }))
    .route("/liquer/q/*query", get(evaluate_query))
    .route("/liquer/submit/*query", get(submit_query))
    .route("/liquer/store/data/*query", get(store_get))
    .route("/liquer/web/*query", get(web_store_get))
    .with_state(shared_state)
    ;


    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}