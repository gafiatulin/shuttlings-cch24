use axum::extract::Path;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, StatusCode};
use leaky_bucket::RateLimiter;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct State {
    limiter: Mutex<RateLimiter>,
}

impl Default for State {
    fn default() -> Self {
        State {
            limiter: Mutex::new(State::new_limiter()),
        }
    }
}

impl State {
    fn new_limiter() -> RateLimiter {
        RateLimiter::builder()
            .initial(5)
            .max(5)
            .interval(Duration::from_millis(1000))
            .build()
    }

    fn reset(&self) {
        let mut rl = self.limiter.lock().unwrap();
        *rl = State::new_limiter();
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UsReq {
    #[serde(skip_serializing_if = "Option::is_none")]
    liters: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gallons: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UkReq {
    #[serde(skip_serializing_if = "Option::is_none")]
    litres: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pints: Option<f32>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Request {
    Us(UsReq),
    Uk(UkReq),
}

impl Request {
    fn parse(body: &str) -> Result<Self, String> {
        match serde_json::from_str::<UsReq>(body) {
            Ok(us) if us.liters.is_some() ^ us.gallons.is_some() => Ok(Request::Us(us)),
            _ => match serde_json::from_str::<UkReq>(body) {
                Ok(uk) if uk.litres.is_some() ^ uk.pints.is_some() => Ok(Request::Uk(uk)),
                _ => Err("Invalid request".to_string()),
            },
        }
    }

    fn convert(&self) -> Request {
        match self {
            Request::Us(us) => Request::Us(UsReq {
                liters: us.gallons.map(|g| g * 3.7854118),
                gallons: us.liters.map(|l| l / 3.7854118),
            }),
            Request::Uk(uk) => Request::Uk(UkReq {
                litres: uk.pints.map(|p| p / 1.759754),
                pints: uk.litres.map(|l| l * 1.759754),
            }),
        }
    }
}

pub async fn milk(
    milk_factory: Arc<State>,
    Path(op): Path<String>,
    headers: HeaderMap,
    body: String,
) -> (StatusCode, String) {
    match op.as_str() {
        "milk" => {
            if milk_factory.limiter.lock().unwrap().try_acquire(1) {
                match headers.get(CONTENT_TYPE) {
                    Some(ct) if ct == "application/json" => match Request::parse(&body) {
                        Ok(request) => {
                            let converted = request.convert();
                            (StatusCode::OK, serde_json::to_string(&converted).unwrap())
                        }
                        _ => (StatusCode::BAD_REQUEST, "".to_string()),
                    },
                    _ => (StatusCode::OK, "Milk withdrawn\n".to_string()),
                }
            } else {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    "No milk available\n".to_string(),
                )
            }
        }
        "refill" => {
            milk_factory.reset();
            (StatusCode::OK, "".to_string())
        }
        _ => (StatusCode::BAD_REQUEST, "".to_string()),
    }
}
