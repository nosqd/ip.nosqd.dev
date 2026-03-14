use axum::http::header::{ACCEPT, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{Html, IntoResponse, Response};
use axum::{Json, Router, extract::State, routing::get};
use axum_client_ip::XRealIp as ClientIp;
use maxminddb::{Reader, geoip2};
use serde::Serialize;
use std::env;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use woothee::parser::Parser;

struct AppState {
    city: Reader<maxminddb::Mmap>,
    asn: Reader<maxminddb::Mmap>,
}

#[derive(Serialize)]
struct IpResponse {
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

    let cors_layer = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin("http://localhost:4001".parse::<HeaderValue>().unwrap())
        .allow_headers([ACCEPT, CONTENT_TYPE]);
    let app = Router::new()
        .route("/", get(root))
        .with_state(state)
        .layer(cors_layer);

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    println!("Listening on port {}", port);
    axum::serve(listener, app).await.unwrap();
}

async fn root(
    ClientIp(ip): ClientIp,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
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

    let data = IpResponse {
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
    };

    let accept_header = headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let ua_string = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let parser = Parser::new();
    let is_browser = parser
        .parse(ua_string)
        .map(|result| {
            matches!(
                result.category,
                "pc" | "smartphone" | "mobilephone" | "crawler"
            )
        })
        .unwrap_or(false);

    if accept_header.contains("application/json") {
        Json(data).into_response()
    } else if is_browser || accept_header.contains("text/html") {
        Html(render_html(data)).into_response()
    } else {
        Json(data).into_response()
    }
}

fn render_html(data: IpResponse) -> String {
    format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>nosqd's IP info</title>
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <meta name="title" content="nosqd's IP info">
            <meta name="description" content="A blazingly fast IP information service written in Rust.">
            <meta name="theme-color" content="\#485588">

            <meta property="og:type" content="website">
            <meta property="og:url" content="https://ip.nosqd.dev/">
            <meta property="og:title" content="nosqd's IP info">
            <meta property="og:description" content="A blazingly fast IP information service written in Rust.">

            <meta property="twitter:card" content="summary">
            <meta property="twitter:title" content="nosqd's IP info">
            <meta property="twitter:description" content="A blazingly fast IP information service written in Rust.">
            <style>
                body {{ background: #282828; color: #ebdbb2; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; font-size: 24px; }}
            </style>
        </head>
        <body class="flex flex-col items-center justify-center min-h-screen p-4">
        <pre>
> Hello from nosqd's IP information service.
<br>
> IP: {ip}
> Location: {flag} {city}, {country}
> ASN: AS{asn} ({asn_org})
<br>
> Try: <code>curl -L ip.nosqd.dev</code>
> Built with Rust & Nix • <a href="https://github.com/nosqd/ip.nosqd.dev" style="text-decoration: none; color: #458588;">GitHub</a>
        </pre>
        </body>
        </html>
        "#,
        ip = data.ip,
        flag = data.flag.unwrap_or_default(),
        city = data.city.unwrap_or_else(|| "Unknown".to_string()),
        country = data.country.unwrap_or_else(|| "Unknown".to_string()),
        asn = data.asn.unwrap_or(0),
        asn_org = data.asn_org.unwrap_or_else(|| "Unknown".to_string()),
    )
}
