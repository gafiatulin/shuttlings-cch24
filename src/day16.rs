use axum::extract::Path;
use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::{HeaderMap, StatusCode};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde_json::Value;

const SECRET: &str = "secret";
const SANTA_PUBLIC_KEY_PEM: &str = include_str!("../keys/day16_santa_public_key.pem");

pub async fn unwrap(Path(op): Path<String>, headers: HeaderMap) -> (StatusCode, String) {
    if op == "unwrap" {
        let maybe_gift_cookie = headers
            .get_all(COOKIE)
            .into_iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|value| value.trim().split(';'))
            .filter_map(|cookie| cookie.split_once('='))
            .find(|(key, _)| *key == "gift")
            .map(|(_, value)| value);

        let validation: Validation = {
            let mut validation = Validation::new(Algorithm::HS256);
            validation.set_required_spec_claims::<String>(&[]);
            validation
        };

        let maybe_token = maybe_gift_cookie.and_then(|cookie| {
            decode::<Value>(
                cookie,
                &DecodingKey::from_secret(SECRET.as_ref()),
                &validation,
            )
            .ok()
        });

        if let Some(token) = maybe_token {
            (StatusCode::OK, token.claims.to_string())
        } else {
            (StatusCode::BAD_REQUEST, String::new())
        }
    } else {
        (StatusCode::NOT_FOUND, String::new())
    }
}

pub async fn jwt(Path(op): Path<String>, body: String) -> (StatusCode, HeaderMap, String) {
    match op.as_str() {
        "wrap" => {
            let payload = serde_json::from_str::<Value>(&body).unwrap();
            let token = encode(
                &Header::default(),
                &payload,
                &EncodingKey::from_secret(SECRET.as_ref()),
            )
            .unwrap();
            let mut headers = HeaderMap::new();
            headers.append(SET_COOKIE, format!("gift={}", token).parse().unwrap());
            (StatusCode::OK, headers, String::new())
        }
        "decode" => {
            let validation = {
                let mut validation = Validation::new(Algorithm::RS256);
                validation.set_required_spec_claims::<String>(&[]);
                validation.algorithms = vec![Algorithm::RS256, Algorithm::RS512];
                validation
            };
            match decode::<Value>(
                &body,
                &DecodingKey::from_rsa_pem(SANTA_PUBLIC_KEY_PEM.as_ref()).unwrap(),
                &validation,
            ) {
                Ok(token) => (StatusCode::OK, HeaderMap::new(), token.claims.to_string()),
                Err(err) => {
                    if let ErrorKind::InvalidSignature = err.kind() {
                        (StatusCode::UNAUTHORIZED, HeaderMap::new(), String::new())
                    } else {
                        (StatusCode::BAD_REQUEST, HeaderMap::new(), String::new())
                    }
                }
            }
        }
        _ => (StatusCode::NOT_FOUND, HeaderMap::new(), String::new()),
    }
}
