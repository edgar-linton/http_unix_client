use std::{fs, io, path::PathBuf, time::Duration};

use actix_web::{
    App, FromRequest, Handler, HttpServer, Responder, http::Method, rt::task::JoinHandle, web,
};

pub struct TestServer {
    handle: JoinHandle<()>,
    socket_path: PathBuf,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.handle.abort();
        if let Err(e) = fs::remove_file(&self.socket_path) {
            eprintln!("Failed to delete socket file: {e}");
        }
    }
}

/// Setup server and return socket path
pub async fn setup_test_server<F, Args>(
    socket: &str,
    route: &str,
    method: Method,
    handler: F,
) -> io::Result<TestServer>
where
    F: Handler<Args> + Send + Sync + Clone + 'static,
    Args: FromRequest + 'static,
    F::Output: Responder + 'static,
{
    let socket_path = PathBuf::from(format!("/tmp/{socket}.socket"));

    if socket_path.exists() {
        fs::remove_file(&socket_path)?;
    }

    let route = route.to_string();
    let server = HttpServer::new(move || {
        App::new().route(&route, web::method(method.clone()).to(handler.clone()))
    })
    .bind_uds(&socket_path)?
    .run();

    let join_handle = actix_web::rt::spawn(async move {
        if let Err(e) = server.await {
            dbg!(e);
        }
    });

    actix_web::rt::time::sleep(Duration::from_millis(300)).await;

    Ok(TestServer {
        handle: join_handle,
        socket_path,
    })
}
