use axum::extract::{Path, Query};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Uuid;
use sqlx::FromRow;
use sqlx::PgPool;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct QuoteRequest {
    author: String,
    quote: String,
}

#[derive(FromRow, Clone, Serialize)]
struct Quote {
    id: Uuid,
    author: String,
    quote: String,
    created_at: DateTime<Utc>,
    version: i32,
}

#[derive(Serialize)]
struct Page {
    quotes: Vec<Quote>,
    page: u64,
    next_token: Option<String>,
}

pub async fn get_route(
    pool: PgPool,
    Path(op): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, String) {
    match split_path_with_uuid(&op) {
        Some(("cite", None)) => (StatusCode::BAD_REQUEST, "".to_string()),
        Some(("cite", id)) => {
            let result = sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1;")
                .bind(id)
                .fetch_optional(&pool)
                .await
                .unwrap();

            quote_or_not_found(result)
        }
        None if op == "list" => match params.get("token") {
            Some(token) => {
                let page_part = token.chars().take_while(|c| c != &'0').collect::<String>();
                let ts_part = token
                    .chars()
                    .skip_while(|c| c != &'0')
                    .skip_while(|c| c == &'0')
                    .collect::<String>();

                if page_part.is_empty()
                    || ts_part.is_empty()
                    || page_part.len() > 11
                    || ts_part.len() > 11
                {
                    return (StatusCode::BAD_REQUEST, "".to_string());
                }

                match (
                    alphanumeric_to_num(&page_part),
                    alphanumeric_to_num(&ts_part),
                ) {
                    (Some(current_page), Some(ts_millis)) => {
                        if let Some(ts) = DateTime::<Utc>::from_timestamp_millis(ts_millis as i64) {
                            let result = sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE created_at >= $1 ORDER BY created_at LIMIT 4;")
                                .bind(ts)
                                .fetch_all(&pool)
                                .await
                                .unwrap();
                            (
                                StatusCode::OK,
                                serde_json::to_string(&mk_pagination(current_page, result))
                                    .unwrap(),
                            )
                        } else {
                            (StatusCode::BAD_REQUEST, "".to_string())
                        }
                    }
                    _ => (StatusCode::BAD_REQUEST, "".to_string()),
                }
            }
            None => {
                let result =
                    sqlx::query_as::<_, Quote>("SELECT * FROM quotes ORDER BY created_at LIMIT 4;")
                        .fetch_all(&pool)
                        .await
                        .unwrap();
                (
                    StatusCode::OK,
                    serde_json::to_string(&mk_pagination(1, result)).unwrap(),
                )
            }
        },
        _ => (StatusCode::NOT_FOUND, "".to_string()),
    }
}

pub async fn post_route(
    pool: PgPool,
    Path(op): Path<String>,
    body: String,
) -> (StatusCode, String) {
    match op.as_str() {
        "reset" => {
            sqlx::query("TRUNCATE quotes;")
                .execute(&pool)
                .await
                .unwrap();
            (StatusCode::OK, "".to_string())
        }
        "draft" => {
            let request = serde_json::from_str::<QuoteRequest>(&body).unwrap();
            let id = Uuid::new_v4();
            let result = sqlx::query_as::<_, Quote>(
                "INSERT INTO quotes (id, author, quote) VALUES ($1, $2, $3) RETURNING *;",
            )
            .bind(id)
            .bind(request.author)
            .bind(request.quote)
            .fetch_one(&pool)
            .await
            .unwrap();

            (StatusCode::CREATED, serde_json::to_string(&result).unwrap())
        }
        _ => (StatusCode::NOT_FOUND, "".to_string()),
    }
}

pub async fn put_route(pool: PgPool, Path(op): Path<String>, body: String) -> (StatusCode, String) {
    match split_path_with_uuid(&op) {
        Some(("undo", None)) => (StatusCode::BAD_REQUEST, "".to_string()),
        Some(("undo", Some(id))) => {
            let request = serde_json::from_str::<QuoteRequest>(&body).unwrap();
            let result = sqlx::query_as::<_, Quote>("UPDATE quotes SET author = $1, quote = $2, version = version + 1 WHERE id = $3 RETURNING *;")
                .bind(request.author)
                .bind(request.quote)
                .bind(id)
                .fetch_optional(&pool)
                .await
                .unwrap();

            quote_or_not_found(result)
        }
        _ => (StatusCode::NOT_FOUND, "".to_string()),
    }
}

pub async fn delete_route(pool: PgPool, Path(op): Path<String>) -> (StatusCode, String) {
    match split_path_with_uuid(&op) {
        Some(("remove", None)) => (StatusCode::BAD_REQUEST, "".to_string()),
        Some(("remove", Some(id))) => {
            let result =
                sqlx::query_as::<_, Quote>("DELETE FROM quotes WHERE id = $1 RETURNING *;")
                    .bind(id)
                    .fetch_optional(&pool)
                    .await
                    .unwrap();

            quote_or_not_found(result)
        }
        _ => (StatusCode::NOT_FOUND, "".to_string()),
    }
}

fn quote_or_not_found(quote: Option<Quote>) -> (StatusCode, String) {
    match quote {
        Some(quote) => (StatusCode::OK, serde_json::to_string(&quote).unwrap()),
        None => (StatusCode::NOT_FOUND, "".to_string()),
    }
}

fn split_path_with_uuid(str: &str) -> Option<(&str, Option<Uuid>)> {
    str.split_once('/')
        .map(|(op, id)| (op, Uuid::parse_str(id).ok()))
}

fn num_to_alphanumeric(num: u64) -> String {
    let mut num = num;
    let mut result = String::new();

    while num > 0 {
        let rm = num % 61;
        let c = match rm {
            0..9 => (rm + 49) as u8,
            9..35 => (rm + 56) as u8,
            35..61 => (rm + 62) as u8,
            _ => unreachable!(),
        };
        result.push(c as char);
        num /= 61;
    }

    result.chars().rev().collect()
}

fn alphanumeric_to_num(s: &str) -> Option<u64> {
    let mut num = 0;
    for c in s.chars() {
        let n = match c {
            '1'..='9' => c as u64 - 49,
            'A'..='Z' => c as u64 - 56,
            'a'..='z' => c as u64 - 62,
            _ => return None,
        };
        num = num * 61 + n;
    }
    Some(num)
}

fn mk_pagination(current_page: u64, results: Vec<Quote>) -> Page {
    let next_token = if results.len() == 4 {
        let next_ts = results.last().unwrap().created_at.timestamp_millis();
        let token_page = num_to_alphanumeric(current_page + 1);
        let token_ts = num_to_alphanumeric(next_ts as u64);
        Some(format!(
            "{}{}{}",
            token_page,
            "0".repeat(16 - token_page.len() - token_ts.len()),
            token_ts
        ))
    } else {
        None
    };

    Page {
        quotes: results.iter().take(3).cloned().collect(),
        page: current_page,
        next_token,
    }
}
