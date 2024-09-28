use crate::{PORT_HTTP, PORT_HTTPS};
use axum::{extract::Host, handler::HandlerWithoutStateExt, http::Uri, response::Redirect};
use openssl::{
    asn1::Asn1Time,
    error::ErrorStack,
    hash::MessageDigest,
    nid::Nid,
    pkey::{PKey, Private},
    rsa::Rsa,
    x509::{X509Name, X509},
};
use reqwest::StatusCode;
use std::net::SocketAddr;

pub fn generate_ssl() -> Result<(X509, PKey<Private>), ErrorStack> {
    let rsa = Rsa::generate(2048)?;
    let pkey = PKey::from_rsa(rsa)?;
    let mut cert = X509::builder()?;
    cert.set_pubkey(pkey.as_ref())?;
    let mut name = X509Name::builder()?;
    name.append_entry_by_nid(Nid::COUNTRYNAME, "PL")?;
    name.append_entry_by_nid(Nid::STATEORPROVINCENAME, "Mazowieckie")?;
    name.append_entry_by_nid(Nid::LOCALITYNAME, "Warsaw")?;
    name.append_entry_by_nid(Nid::ORGANIZATIONNAME, "Home API")?;
    name.append_entry_by_nid(Nid::ORGANIZATIONALUNITNAME, "IT")?;
    name.append_entry_by_nid(Nid::COMMONNAME, "Home API")?;
    let name = name.build();
    cert.set_subject_name(name.as_ref())?;
    cert.set_issuer_name(name.as_ref())?;
    let nbf = Asn1Time::days_from_now(0)?;
    let naf = Asn1Time::days_from_now(365)?;
    cert.set_not_before(nbf.as_ref())?;
    cert.set_not_after(naf.as_ref())?;
    cert.sign(&pkey, MessageDigest::sha256())?;

    Ok((cert.build(), pkey))
}

pub fn start_https_redirect_server() {
    tokio::spawn(async {
        let addr = SocketAddr::from(([0, 0, 0, 0], PORT_HTTP));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        tracing::info!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, redirect.into_make_service())
            .await
            .unwrap();
    });
}

async fn redirect(Host(host): Host, uri: Uri) -> Result<Redirect, StatusCode> {
    Ok(Redirect::permanent(
        &make_https(host, uri)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .to_string(),
    ))
}

fn make_https(host: String, uri: Uri) -> Result<Uri, Box<dyn std::error::Error>> {
    let mut parts = uri.into_parts();
    parts.scheme = Some(axum::http::uri::Scheme::HTTPS);
    if parts.path_and_query.is_none() {
        parts.path_and_query = Some("/".parse().unwrap());
    }

    let https_host = host.replace(&PORT_HTTP.to_string(), &PORT_HTTPS.to_string());
    parts.authority = Some(https_host.parse()?);
    Ok(Uri::from_parts(parts)?)
}
