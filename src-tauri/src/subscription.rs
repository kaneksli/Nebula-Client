use crate::geo::{flag_and_country_from_remark, lookup_country};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedServer {
    pub id: String,
    pub country: String,
    pub flag: String,
    pub protocol: String,
    pub address: String,
    pub port: u16,
    pub ping: u32,
    pub lat: f64,
    pub lng: f64,
    pub raw_outbound: serde_json::Value,
}

fn decode_b64_loose(s: &str) -> Option<Vec<u8>> {
    STANDARD
        .decode(s)
        .or_else(|_| URL_SAFE_NO_PAD.decode(s.trim_end_matches('=')))
        .ok()
}

/// Fetches a subscription URL and returns every server link found. Handles
/// both a plain newline-separated link list and the common "whole body is
/// base64" subscription format.
pub async fn fetch_links(url: &str) -> Result<Vec<String>> {
    let body = reqwest::get(url).await?.text().await?;
    let decoded = decode_b64_loose(body.trim())
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or(body);

    Ok(decoded
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| {
            l.starts_with("vless://")
                || l.starts_with("vmess://")
                || l.starts_with("trojan://")
                || l.starts_with("hysteria2://")
                || l.starts_with("hy2://")
                || l.starts_with("ss://")
        })
        .collect())
}

pub fn parse_link(link: &str) -> Result<ParsedServer> {
    if let Some(rest) = link.strip_prefix("vless://") {
        parse_uri_style(rest, "vless")
    } else if let Some(rest) = link.strip_prefix("trojan://") {
        parse_uri_style(rest, "trojan")
    } else if let Some(rest) = link.strip_prefix("hysteria2://") {
        parse_uri_style(rest, "hysteria2")
    } else if let Some(rest) = link.strip_prefix("hy2://") {
        parse_uri_style(rest, "hysteria2")
    } else if let Some(rest) = link.strip_prefix("vmess://") {
        parse_vmess(rest)
    } else if let Some(rest) = link.strip_prefix("ss://") {
        parse_ss(rest)
    } else {
        Err(anyhow!("unsupported link scheme: {link}"))
    }
}

/// vless / trojan / hysteria2 all share the same shape:
/// scheme://uuid-or-password@host:port?query#remark
fn parse_uri_style(rest: &str, protocol: &str) -> Result<ParsedServer> {
    let full = format!("proto://{rest}");
    let parsed = Url::parse(&full)?;

    let userinfo = parsed.username().to_string();
    let host = parsed.host_str().ok_or_else(|| anyhow!("missing host"))?.to_string();
    let port = parsed.port().ok_or_else(|| anyhow!("missing port"))?;
    let remark = parsed
        .fragment()
        .map(|f| urlencoding_decode(f))
        .unwrap_or_else(|| host.clone());

    let query: HashMap<String, String> = parsed.query_pairs().into_owned().collect();
    let sni = query.get("sni").or(query.get("peer")).cloned().unwrap_or_else(|| host.clone());
    let security = query.get("security").cloned().unwrap_or_default();

    let outbound = match protocol {
        "vless" => json!({
            "type": "vless",
            "tag": "proxy",
            "server": host,
            "server_port": port,
            "uuid": userinfo,
            "flow": query.get("flow").cloned().unwrap_or_default(),
            "tls": {
                "enabled": true,
                "server_name": sni,
                "reality": if security == "reality" {
                    json!({
                        "enabled": true,
                        "public_key": query.get("pbk").cloned().unwrap_or_default(),
                        "short_id": query.get("sid").cloned().unwrap_or_default()
                    })
                } else { json!(null) },
                "utls": { "enabled": true, "fingerprint": query.get("fp").cloned().unwrap_or_else(|| "chrome".into()) }
            },
            "transport": transport_block(&query)
        }),
        "trojan" => json!({
            "type": "trojan",
            "tag": "proxy",
            "server": host,
            "server_port": port,
            "password": userinfo,
            "tls": { "enabled": true, "server_name": sni }
        }),
        "hysteria2" => json!({
            "type": "hysteria2",
            "tag": "proxy",
            "server": host,
            "server_port": port,
            "password": userinfo,
            "tls": { "enabled": true, "server_name": sni, "insecure": query.get("insecure").map(|v| v=="1").unwrap_or(false) }
        }),
        _ => unreachable!(),
    };

    let (flag, country_hint) = flag_and_country_from_remark(&remark);
    let country = country_hint.unwrap_or(remark);

    Ok(ParsedServer {
        id: format!("srv-{}", uuid::Uuid::new_v4()),
        country,
        flag,
        protocol: protocol_label(protocol, &query),
        address: host,
        port,
        ping: 0,
        lat: 0.0,
        lng: 0.0,
        raw_outbound: outbound,
    })
}

fn transport_block(query: &HashMap<String, String>) -> serde_json::Value {
    match query.get("type").map(|s| s.as_str()) {
        Some("ws") => json!({ "type": "ws", "path": query.get("path").cloned().unwrap_or_default() }),
        Some("grpc") => json!({ "type": "grpc", "service_name": query.get("serviceName").cloned().unwrap_or_default() }),
        Some("xhttp") | Some("splithttp") => json!({ "type": "httpupgrade", "path": query.get("path").cloned().unwrap_or_default() }),
        _ => json!(null),
    }
}

fn protocol_label(protocol: &str, query: &HashMap<String, String>) -> String {
    let net = query.get("type").cloned().unwrap_or_default();
    let security = query.get("security").cloned().unwrap_or_default();
    match (protocol, net.as_str(), security.as_str()) {
        ("vless", "xhttp", "reality") | ("vless", "splithttp", "reality") => "XHTTP Reality".into(),
        ("vless", _, "reality") => "VLESS Reality".into(),
        ("vless", _, _) => "VLESS".into(),
        ("trojan", _, _) => "Trojan".into(),
        ("hysteria2", _, _) => "Hysteria2".into(),
        _ => protocol.to_uppercase(),
    }
}

/// vmess:// carries a base64 JSON blob instead of URI-encoded query params.
fn parse_vmess(rest: &str) -> Result<ParsedServer> {
    let bytes = decode_b64_loose(rest).ok_or_else(|| anyhow!("invalid vmess base64"))?;
    let v: serde_json::Value = serde_json::from_slice(&bytes)?;

    let host = v["add"].as_str().unwrap_or_default().to_string();
    let port: u16 = v["port"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .or_else(|| v["port"].as_u64().map(|p| p as u16))
        .ok_or_else(|| anyhow!("missing vmess port"))?;
    let remark = v["ps"].as_str().unwrap_or(&host).to_string();
    let tls_on = v["tls"].as_str().unwrap_or("") == "tls";

    let outbound = json!({
        "type": "vmess",
        "tag": "proxy",
        "server": host,
        "server_port": port,
        "uuid": v["id"].as_str().unwrap_or_default(),
        "security": "auto",
        "alter_id": v["aid"].as_u64().unwrap_or(0),
        "tls": { "enabled": tls_on, "server_name": v["sni"].as_str().unwrap_or(&host) },
        "transport": match v["net"].as_str() {
            Some("ws") => json!({ "type": "ws", "path": v["path"].as_str().unwrap_or("") }),
            Some("grpc") => json!({ "type": "grpc", "service_name": v["path"].as_str().unwrap_or("") }),
            _ => json!(null),
        }
    });

    let (flag, country_hint) = flag_and_country_from_remark(&remark);
    let country = country_hint.unwrap_or(remark);

    Ok(ParsedServer {
        id: format!("srv-{}", uuid::Uuid::new_v4()),
        country,
        flag,
        protocol: "VMess".into(),
        address: host,
        port,
        ping: 0,
        lat: 0.0,
        lng: 0.0,
        raw_outbound: outbound,
    })
}

/// ss://base64(method:password)@host:port#remark  (also tolerates the fully
/// base64-encoded legacy form ss://base64(method:password@host:port))
fn parse_ss(rest: &str) -> Result<ParsedServer> {
    let (main, remark) = match rest.split_once('#') {
        Some((m, r)) => (m, urlencoding_decode(r)),
        None => (rest, String::new()),
    };

    let (userinfo, hostport) = if let Some((u, hp)) = main.split_once('@') {
        (u.to_string(), hp.to_string())
    } else {
        // fully-encoded legacy form
        let decoded = decode_b64_loose(main)
            .and_then(|b| String::from_utf8(b).ok())
            .ok_or_else(|| anyhow!("invalid legacy ss:// payload"))?;
        let (u, hp) = decoded.split_once('@').ok_or_else(|| anyhow!("malformed ss:// payload"))?;
        (u.to_string(), hp.to_string())
    };

    let creds = decode_b64_loose(&userinfo)
        .and_then(|b| String::from_utf8(b).ok())
        .unwrap_or(userinfo);
    let (method, password) = creds.split_once(':').ok_or_else(|| anyhow!("malformed ss:// credentials"))?;

    let (host, port_str) = hostport.rsplit_once(':').ok_or_else(|| anyhow!("malformed ss:// host:port"))?;
    let port: u16 = port_str
        .trim_end_matches(|c: char| !c.is_ascii_digit())
        .parse()
        .map_err(|_| anyhow!("invalid ss:// port"))?;

    let outbound = json!({
        "type": "shadowsocks",
        "tag": "proxy",
        "server": host,
        "server_port": port,
        "method": method,
        "password": password
    });

    let remark_or_host = if remark.is_empty() { host.to_string() } else { remark };
    let (flag, country_hint) = flag_and_country_from_remark(&remark_or_host);
    let country = country_hint.unwrap_or(remark_or_host);

    Ok(ParsedServer {
        id: format!("srv-{}", uuid::Uuid::new_v4()),
        country,
        flag,
        protocol: "Shadowsocks".into(),
        address: host.to_string(),
        port,
        ping: 0,
        lat: 0.0,
        lng: 0.0,
        raw_outbound: outbound,
    })
}

fn urlencoding_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8_lossy()
        .to_string()
}

/// Fills in lat/lng once the country is known (parsed from the remark, or
/// left as a best-effort guess); falls back to (0, 0) when unrecognized so
/// the globe simply doesn't fly anywhere odd instead of guessing wrong.
pub fn attach_coordinates(server: &mut ParsedServer) {
    if let Some((lat, lng)) = lookup_country(&server.country) {
        server.lat = lat;
        server.lng = lng;
    }
}
