use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, StatusCode};
use cargo_manifest::{Manifest, MaybeInherited};
use serde::Deserialize;
use toml::Value;

enum Format {
    Json,
    Yaml,
    Toml,
}

#[derive(Debug, Deserialize)]
struct Order {
    item: String,
    quantity: u64,
}

#[derive(Deserialize)]
struct OrderList {
    orders: Vec<Value>,
}

pub async fn manifest(headers: HeaderMap, body: String) -> (StatusCode, String) {
    if let Some(format) = get_format(&headers) {
        match parse_orders(&body, format) {
            Ok(orders) => {
                if orders.is_empty() {
                    (StatusCode::NO_CONTENT, String::new())
                } else {
                    let response_body = orders
                        .iter()
                        .map(|order| format!("{}: {}", order.item, order.quantity))
                        .collect::<Vec<_>>()
                        .join("\n");
                    (StatusCode::OK, response_body)
                }
            }
            Err(msg) => (StatusCode::BAD_REQUEST, msg),
        }
    } else {
        (StatusCode::UNSUPPORTED_MEDIA_TYPE, String::new())
    }
}

fn get_format(headers: &HeaderMap) -> Option<Format> {
    match headers.get(CONTENT_TYPE) {
        Some(ct) if ct == "application/toml" => Some(Format::Toml),
        Some(ct) if ct == "application/json" => Some(Format::Json),
        Some(ct) if ct == "application/yaml" => Some(Format::Yaml),
        _ => None,
    }
}

fn parse_orders(body: &str, format: Format) -> Result<Vec<Order>, String> {
    let manifest = parse_manifest(body, format).map_err(|e| e.to_string());

    let package = manifest?.package.unwrap();

    let maybe_keywords = package
        .keywords
        .unwrap_or(MaybeInherited::Local(Vec::new()));

    match maybe_keywords {
        MaybeInherited::Local(keywords) if keywords.contains(&"Christmas 2024".to_string()) => {
            match package.metadata {
                Some(metadata) => match metadata.try_into::<OrderList>() {
                    Ok(ol) => {
                        let orders: Vec<Order> = ol
                            .orders
                            .iter()
                            .flat_map(|v| v.clone().try_into::<Order>().ok())
                            .collect();
                        Ok(orders)
                    }
                    Err(_) => Ok(Vec::new()),
                },
                None => Ok(Vec::new()),
            }
        }
        _ => Err("Magic keyword not provided".to_string()),
    }
}

fn parse_manifest(body: &str, format: Format) -> Result<Manifest, &str> {
    match format {
        Format::Json => serde_json::from_str::<Manifest>(body).map_err(|_| "Invalid manifest"),
        Format::Yaml => serde_yml::from_str::<Manifest>(body).map_err(|_| "Invalid manifest"),
        Format::Toml => toml::from_str::<Manifest>(body).map_err(|_| "Invalid manifest"),
    }
}
