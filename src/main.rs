mod day0;
mod day2;
mod day5;
mod day9;

use axum::extract::Path;
use axum::{routing, Router};
use std::sync::Arc;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let day9_state = Arc::new(day9::State::default());

    let router = Router::new()
        .route("/", routing::get(day0::hello_bird))
        .route("/-1/seek", routing::get(day0::seek))
        .route("/2/*v_op", routing::get(day2::route))
        .route("/5/manifest", routing::post(day5::manifest))
        .route(
            "/9/:op",
            routing::post({
                let shared_state = Arc::clone(&day9_state);
                move |Path(op), headers, body| day9::milk(shared_state, Path(op), headers, body)
            }),
        );
    Ok(router.into())
}
