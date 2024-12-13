mod day0;
mod day12;
mod day2;
mod day5;
mod day9;

use axum::extract::Path;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let day9_state = Arc::new(day9::State::default());
    let day12_state = Arc::new(day12::State::default());

    let router = Router::new()
        .route("/", get(day0::hello_bird))
        .route("/-1/seek", get(day0::seek))
        .route("/2/*v_op", get(day2::route))
        .route("/5/manifest", post(day5::manifest))
        .route(
            "/9/:op",
            post({
                let shared_state = Arc::clone(&day9_state);
                move |Path(op), headers, body| day9::milk(shared_state, Path(op), headers, body)
            }),
        )
        .route(
            "/12/*op",
            get({
                let shared_state = Arc::clone(&day12_state);
                move |Path(op)| day12::board(shared_state, Path(op))
            })
            .post({
                let shared_state = Arc::clone(&day12_state);
                move |Path(op)| day12::game(shared_state, Path(op))
            }),
        );
    Ok(router.into())
}
