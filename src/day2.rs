use axum::extract::{Path, Query};
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ops::BitXor;
use std::str::FromStr;

pub async fn route(
    Path(v_op): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> String {
    match v_op.split_once('/') {
        Some(("v6", op)) => route_v6(op.to_string(), params),
        _ => route_v4(v_op, params),
    }
}

fn route_v4(op: String, params: HashMap<String, String>) -> String {
    let from = Ipv4Addr::from_str(params.get("from").unwrap()).unwrap();

    let result = if op == "dest" {
        let key = Ipv4Addr::from_str(params.get("key").unwrap()).unwrap();
        key.octets()
            .iter()
            .zip(from.octets().iter())
            .map(|(k, f)| k.wrapping_add(*f))
            .collect::<Vec<u8>>()
    } else {
        let to = Ipv4Addr::from_str(params.get("to").unwrap()).unwrap();
        to.octets()
            .iter()
            .zip(from.octets().iter())
            .map(|(t, f)| t.wrapping_sub(*f))
            .collect::<Vec<u8>>()
    };

    let octets: [u8; 4] = result.try_into().unwrap();

    Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]).to_string()
}

fn route_v6(op: String, params: HashMap<String, String>) -> String {
    let from = Ipv6Addr::from_str(params.get("from").unwrap()).unwrap();

    let other = if op == "dest" {
        params.get("key").unwrap()
    } else {
        params.get("to").unwrap()
    };

    Ipv6Addr::from(
        Ipv6Addr::from_str(other)
            .unwrap()
            .to_bits()
            .bitxor(from.to_bits()),
    )
    .to_string()
}
