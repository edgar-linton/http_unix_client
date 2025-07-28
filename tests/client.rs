use actix_web::HttpResponse;
use actix_web::http::Method;
use http_unix_client::Client;
use std::io;

mod server;

#[actix_web::test]
async fn response_text() -> io::Result<()> {
    let server = server::setup_test_server("text", "/test", Method::GET, async || {
        HttpResponse::Ok().body("Hello, World!")
    })
    .await?;

    let resp = Client::new()
        .get("/tmp/text.socket", "/test")
        .send()
        .await
        .map_err(io::Error::other)?;

    assert_eq!(resp.content_length(), Some(13));
    assert_eq!(&resp.text().await.unwrap(), "Hello, World!");

    drop(server);

    Ok(())
}

#[actix_web::test]
async fn response_bytes() -> io::Result<()> {
    let server = server::setup_test_server("bytes", "/test", Method::GET, async || {
        HttpResponse::Ok().body("Hello, World!")
    })
    .await?;

    let resp = Client::new()
        .get("/tmp/bytes.socket", "/test")
        .send()
        .await
        .map_err(io::Error::other)?;

    assert_eq!(resp.content_length(), Some(13));
    assert_eq!(resp.bytes().await.unwrap(), "Hello, World!");

    drop(server);

    Ok(())
}

#[actix_web::test]
#[cfg(feature = "json")]
async fn response_json() -> io::Result<()> {
    let server = server::setup_test_server("json", "/test", Method::GET, async || {
        HttpResponse::Ok().json(("foo", "bar"))
    })
    .await?;

    let resp = Client::new()
        .get("/tmp/json.socket", "/test")
        .send()
        .await
        .map_err(io::Error::other)?;

    assert_eq!(resp.content_length(), Some(13));
    assert_eq!(resp.json::<(String, String)>().await.unwrap().1, "bar");

    drop(server);

    Ok(())
}

#[actix_web::test]
async fn response_header() -> io::Result<()> {
    use http::HeaderValue;

    let server = server::setup_test_server("header", "/test", Method::GET, async || {
        HttpResponse::NoContent()
            .append_header(("Date", "Thu, 01 Jan 1970 00:00:00 GMT"))
            .finish()
    })
    .await?;

    let resp = Client::new()
        .get("/tmp/header.socket", "/test")
        .send()
        .await
        .map_err(io::Error::other)?;

    assert_eq!(
        resp.headers().get("Date"),
        Some(&HeaderValue::from_static("Thu, 01 Jan 1970 00:00:00 GMT"))
    );

    drop(server);

    Ok(())
}

#[actix_web::test]
#[cfg(feature = "cookies")]
async fn response_cookie() -> io::Result<()> {
    use actix_web::cookie::Cookie;
    use std::collections::HashMap;

    let server = server::setup_test_server("cookie", "/test", Method::GET, async || {
        HttpResponse::NoContent()
            .cookie(Cookie::new("token", "jwt123"))
            .cookie(Cookie::new("user", "alice"))
            .finish()
    })
    .await?;

    let resp = Client::new()
        .get("/tmp/cookie.socket", "/test")
        .send()
        .await
        .map_err(io::Error::other)?;

    let cookies: HashMap<String, String> = resp
        .cookies()
        .map(|c| (c.name().to_owned(), c.value().to_owned()))
        .collect::<HashMap<_, _>>();
    assert_eq!(cookies.get("token"), Some(&"jwt123".to_owned()));
    assert_eq!(cookies.get("user"), Some(&"alice".to_owned()));

    drop(server);

    Ok(())
}

#[actix_web::test]
async fn response_fail() -> io::Result<()> {
    let server = server::setup_test_server("fail", "/test", Method::GET, async || {
        HttpResponse::InternalServerError().finish()
    })
    .await?;

    let resp = Client::new()
        .get("/tmp/fail.socket", "/test")
        .send()
        .await
        .map_err(io::Error::other)?;

    assert!(resp.error_for_status().map_err(io::Error::other).is_err());

    drop(server);

    Ok(())
}
