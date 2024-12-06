mod day0;
mod day2;
mod day5;

use axum::{routing, Router};

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", routing::get(day0::hello_bird))
        .route("/-1/seek", routing::get(day0::seek))
        .route("/2/*v_op", routing::get(day2::route))
        .route("/5/manifest", routing::post(day5::manifest));
    Ok(router.into())
}
