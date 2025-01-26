use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use puppydog::service::PuppyService;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to default");

    let mut dog = PuppyService::default();

    dog.register("FrontLeftHip", 0);
    dog.register("FrontLeftThigh", 1);
    dog.register("FrontLeftKnee", 2);

    dog.register("FrontRightHip", 3);
    dog.register("FrontRightThigh", 4);
    dog.register("FrontRightKnee", 5);

    dog.register("BackLeftHip", 6);
    dog.register("BackLeftThigh", 7);
    dog.register("BackLeftKnee", 8);

    dog.register("BackRightHip", 9);
    dog.register("BackRightThigh", 10);
    dog.register("BackRightKnee", 11);

    loop {
        let (socket, _) = listener
            .accept()
            .await
            .expect("Failed to accept connection");

        let io = TokioIo::new(socket);

        let service = dog.clone();
        tokio::spawn(async move {
            if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("Error serving connection: {e}");
            }
        });
    }
}
