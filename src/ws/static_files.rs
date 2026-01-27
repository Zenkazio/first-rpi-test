use axum::{
    body::Body,
    http::{Response, StatusCode, header},
    response::IntoResponse,
};
use include_dir::{Dir, include_dir};

static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

pub async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    // z.B. "/sounds/reset.mp3" -> "sounds/reset.mp3"
    let path = uri.path().trim_start_matches('/');

    // "/" -> index.html
    let path = if path.is_empty() { "index.html" } else { path };

    match ASSETS.get_file(path) {
        Some(file) => {
            let body = Body::from(file.contents());
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(body)
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(format!("File not found: {}", path)))
            .unwrap(),
    }
}
