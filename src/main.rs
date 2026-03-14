use axum::{Json, Router, extract::State, routing::get};
use axum_client_ip::XRealIp as ClientIp;
use maxminddb::{Reader, geoip2};
use serde::Serialize;
use std::env;
use std::sync::Arc;

struct AppState {
    city: Reader<maxminddb::Mmap>,
    asn: Reader<maxminddb::Mmap>,
}

#[derive(Serialize)]
struct Response {
    ip: String,
    city: Option<String>,
    country: Option<String>,
    asn: Option<u32>,
    asn_org: Option<String>,
    flag: Option<String>,
}

#[tokio::main]
async fn main() {
    let city_path = env::var("CITY_DB_PATH").unwrap_or_else(|_| "GeoLite2-City.mmdb".to_string());
    let asn_path = env::var("ASN_DB_PATH").unwrap_or_else(|_| "GeoLite2-ASN.mmdb".to_string());

    let state = Arc::new(AppState {
        city: unsafe { Reader::open_mmap(&city_path).expect("City DB missing") },
        asn: unsafe { Reader::open_mmap(&asn_path).expect("ASN DB missing") },
    });

    let app = Router::new().route("/", get(root)).with_state(state);

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    println!("Listening on port {}", port);
    axum::serve(listener, app).await.unwrap();
}

async fn root(ClientIp(ip): ClientIp, State(state): State<Arc<AppState>>) -> Json<Response> {
    let city_data = state
        .city
        .lookup(ip)
        .ok()
        .and_then(|res| res.decode::<geoip2::City>().ok().flatten());
    let asn_data = state
        .asn
        .lookup(ip)
        .ok()
        .and_then(|res| res.decode::<geoip2::Asn>().ok().flatten());

    let country_code = city_data.as_ref().and_then(|c| c.country.iso_code);

    Json(Response {
        ip: ip.to_string(),

        city: city_data
            .as_ref()
            .and_then(|c| c.city.names.english)
            .map(|s| s.to_string()),

        country: city_data
            .as_ref()
            .and_then(|c| c.country.names.english)
            .map(|s| s.to_string()),

        asn: asn_data.as_ref().and_then(|a| a.autonomous_system_number),
        asn_org: asn_data
            .as_ref()
            .and_then(|a| a.autonomous_system_organization)
            .map(|s| s.to_string()),

        flag: country_code.map(|code| {
            code.chars()
                .filter_map(|ch| char::from_u32(ch as u32 + 127397))
                .collect()
        }),
    })
}
