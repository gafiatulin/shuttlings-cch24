use askama::Template;
use axum::extract::{Multipart, Path};
use axum::http::StatusCode;
use serde::Deserialize;
use std::error::Error;

#[derive(Template)]
#[template(path = "star.html")]
struct StarTemplate<'a> {
    class: &'a str,
}

#[derive(Template)]
#[template(path = "present.html")]
struct PresentTemplate<'a> {
    current: &'a str,
    next: &'a str,
}

#[derive(Template)]
#[template(path = "ornament.html")]
struct OrnamentTemplate<'a> {
    class: &'a str,
    next: &'a str,
    n: &'a str,
}

#[derive(Debug)]
struct Element {
    color: String,
    top: u8,
    left: u8,
}

#[derive(Template)]
#[template(path = "styling.html")]
struct StylingTemplate {
    elements: Vec<Element>,
}

#[derive(Deserialize)]
struct CargoLock {
    package: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    checksum: Option<String>,
}

pub async fn get_route(Path(op): Path<String>) -> (StatusCode, String) {
    match op.as_str() {
        "star" => {
            let template = StarTemplate { class: "lit" };
            (StatusCode::OK, template.render().unwrap())
        }
        s if s.starts_with("present/") => {
            let current = s.strip_prefix("present/").unwrap();

            if let Some((current, next)) = cycle_colors(current).map(|next| (current, next)) {
                let template = PresentTemplate { current, next };
                (StatusCode::OK, template.render().unwrap())
            } else {
                (StatusCode::IM_A_TEAPOT, "".to_string())
            }
        }
        s if s.starts_with("ornament/") => {
            let rest = s.strip_prefix("ornament/").unwrap();
            if let Some((state, n)) = rest.split_once("/") {
                if let Some(next) = cycle_states(state) {
                    let class = if state == "on" {
                        "ornament on"
                    } else {
                        "ornament"
                    };
                    let template = OrnamentTemplate { class, next, n };
                    (StatusCode::OK, template.render().unwrap())
                } else {
                    (StatusCode::IM_A_TEAPOT, "".to_string())
                }
            } else {
                (StatusCode::NOT_FOUND, "".to_string())
            }
        }
        _ => (StatusCode::NOT_FOUND, "".to_string()),
    }
}

pub async fn post_route(Path(op): Path<String>, body: Multipart) -> (StatusCode, String) {
    match op.as_str() {
        "lockfile" => {
            if let Ok(Some(lockfile)) = get_lockfile(body).await {
                let elements = lockfile
                    .package
                    .iter()
                    .filter_map(|package| package.checksum.clone())
                    .map(checksum_to_element)
                    .collect::<Result<Vec<Element>, _>>();

                if let Ok(elements) = elements {
                    let template = StylingTemplate { elements };
                    (StatusCode::OK, template.render().unwrap())
                } else {
                    (StatusCode::UNPROCESSABLE_ENTITY, "".to_string())
                }
            } else {
                (StatusCode::BAD_REQUEST, "".to_string())
            }
        }
        _ => (StatusCode::NOT_FOUND, "".to_string()),
    }
}

fn cycle_colors(color: &str) -> Option<&str> {
    match color {
        "red" => Some("blue"),
        "blue" => Some("purple"),
        "purple" => Some("red"),
        _ => None,
    }
}

fn cycle_states(state: &str) -> Option<&str> {
    match state {
        "off" => Some("on"),
        "on" => Some("off"),
        _ => None,
    }
}

async fn get_lockfile(mut body: Multipart) -> Result<Option<CargoLock>, Box<dyn Error>> {
    let mut lockfile = None;
    while let Some(field) = body.next_field().await? {
        if let Some("lockfile") = field.name() {
            let data = field.text().await?;
            lockfile = Some(toml::from_str::<CargoLock>(&data)?);
        }
    }
    Ok(lockfile)
}

fn checksum_to_element(checksum: String) -> Result<Element, Box<dyn Error>> {
    if checksum.len() < 10 {
        Err("checksum too short".into())
    } else {
        u32::from_str_radix(&checksum[0..6], 16)?;
        let top = u8::from_str_radix(&checksum[6..8], 16)?;
        let left = u8::from_str_radix(&checksum[8..10], 16)?;
        Ok(Element {
            color: format!("#{}", &checksum[0..6]),
            top,
            left,
        })
    }
}
