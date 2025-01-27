use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use pca9685_rppal::Pca9685;
use puppydog::{msg::PuppyMsg, service::PuppyService};
use tokio::net::TcpListener;

/// Minimum pulse length
const SERVO_MIN: u16 = 150;
/// Maximum pulse length
const SERVO_MAX: u16 = 600;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to default");

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    let mut pca9865 = Pca9685::new().expect("Create new PCA9685");
    pca9865.init().expect("Initialize PCA9685");
    pca9865.set_pwm_freq(50.0).expect("Set frequency to 50hz");

    let mut dog = PuppyService::with_send(tx);

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

    tokio::spawn(async move {
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
    });

    while let Some(msg) = rx.recv().await {
        match msg {
            PuppyMsg::MoveServe(idx, angle) => {
                move_servo(&mut pca9865, idx, angle).expect("Move servo");
            }
        }
    }
}

fn map_angle_to_pulse(angle: u16, servomin: u16, servomax: u16) -> u16 {
    let input_min = 0;
    let input_max = 180;
    servomin + (angle - input_min) * (servomax - servomin) / (input_max - input_min)
}

fn move_servo(pca: &mut Pca9685, idx: u8, angle: u16) -> rppal::i2c::Result<()> {
    let len = map_angle_to_pulse(angle, SERVO_MIN, SERVO_MAX);
    pca.set_pwm(idx, 0, len)?;

    Ok(())
}
