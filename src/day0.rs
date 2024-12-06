use axum::http::header::LOCATION;
use axum::http::{HeaderMap, HeaderValue, StatusCode};

pub async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

pub async fn seek() -> (StatusCode, HeaderMap) {
    let mut map = HeaderMap::new();
    map.append(
        LOCATION,
        HeaderValue::from_str("https://www.youtube.com/watch?v=9Gc4QTqslN4").unwrap(),
    );
    (StatusCode::FOUND, map)
}
