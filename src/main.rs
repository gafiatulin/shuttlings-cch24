mod day0;
mod day12;
mod day16;
mod day19;
mod day2;
mod day23;
mod day5;
mod day9;

use axum::extract::{Path, Query};
use axum::{
    routing::{get, post},
    Router,
};
use shuttle_shared_db::Postgres;
use sqlx::{migrate, PgPool};
use std::sync::Arc;
use tower_http::services::ServeDir;

#[shuttle_runtime::main]
async fn main(#[Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    let day9_state = Arc::new(day9::State::default());
    let day12_state = Arc::new(day12::State::default());

    migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let router = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
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
        )
        .route(
            "/16/:op",
            get(move |Path(op), headers| day16::unwrap(Path(op), headers))
                .post(move |Path(op), body| day16::jwt(Path(op), body)),
        )
        .route(
            "/19/*op",
            get({
                let pool = pool.clone();
                move |Path(op), Query(params)| day19::get_route(pool, Path(op), Query(params))
            })
            .post({
                let pool = pool.clone();
                move |Path(op), body| day19::post_route(pool, Path(op), body)
            })
            .put({
                let pool = pool.clone();
                move |Path(op), body| day19::put_route(pool, Path(op), body)
            })
            .delete({
                let pool = pool.clone();
                move |Path(op)| day19::delete_route(pool, Path(op))
            }),
        )
        .route(
            "/23/*op",
            get(move |Path(op)| day23::get_route(Path(op)))
                .post(move |Path(op), body| day23::post_route(Path(op), body)),
        );
    Ok(router.into())
}
