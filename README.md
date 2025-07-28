# http_unix_client

> Async HTTP client over Unix domain sockets – lightweight, fast, and built on [`hyper`].

[![Crates.io](https://img.shields.io/crates/v/http_unix_client.svg)](https://crates.io/crates/http_unix_client)
[![Documentation](https://docs.rs/http_unix_client/badge.svg)](https://docs.rs/http_unix_client)
[![MIT/Apache-2.0 licensed](https://img.shields.io/crates/l/http_unix_client.svg)](#license)

---

**`unix_http_client`** is an asynchronous HTTP client for communicating with local HTTP servers over Unix domain sockets.  
Inspired by the [`reqwest`] API, but tailored for inter-process communication (IPC) on Unix-based systems.

> 🌀 **Currently async-only** — blocking support like `reqwest::blocking` is not yet implemented.

---

## 🚀 Example

```rust,no_run
use unix_http_client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client
        .get("/tmp/my.socket", "/health")
        .send()
        .await?;

    println!("Status: {}", response.status());
    Ok(())
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
