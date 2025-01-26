use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use puppydog::service::PuppyService;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to default");

    let service = PuppyService::default();

    loop {
        let (socket, _) = listener
            .accept()
            .await
            .expect("Failed to accept connection");

        let io = TokioIo::new(socket);

        let service_clone = service.clone();
        tokio::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .serve_connection(io, service_clone)
                .await
            {
                eprintln!("Error serving connection: {e}");
            }
        });
    }
}
