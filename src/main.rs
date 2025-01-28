use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
#[cfg(feature = "runtime")]
use pca9685_rppal::Pca9685;
use puppydog::{msg::PuppyMsg, service::PuppyService};
use tokio::net::TcpListener;

#[cfg(feature = "runtime")]
/// Minimum pulse length
const SERVO_MIN: u16 = 150;
#[cfg(feature = "runtime")]
/// Maximum pulse length
const SERVO_MAX: u16 = 600;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to default");

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    #[cfg(feature = "runtime")]
    let mut pca9865 = {
        let mut pca9865 = Pca9685::new().expect("Create new PCA9685");
        pca9865.init().expect("Initialize PCA9685");
        pca9865.set_pwm_freq(50.0).expect("Set frequency to 50hz");

        pca9865
    };

    let mut dog = PuppyService::with_send(tx);

    dog.register("FLHip", 0).await;
    dog.register("FLThigh", 1).await;
    dog.register("FLKnee", 2).await;

    dog.register("FRHip", 3).await;
    dog.register("FRThigh", 4).await;
    dog.register("FRKnee", 5).await;

    dog.register("BLHip", 6).await;
    dog.register("BLThigh", 7).await;
    dog.register("BLKnee", 8).await;

    dog.register("BRHip", 9).await;
    dog.register("BRThigh", 10).await;
    dog.register("BRKnee", 11).await;

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
                #[cfg(not(feature = "runtime"))]
                println!("Moving servo {idx} to {angle} degrees");

                #[cfg(feature = "runtime")]
                move_servo(&mut pca9865, idx, angle).expect("Move servo");
            }
        }
    }
}

#[cfg(feature = "runtime")]
fn map_angle_to_pulse(angle: u16, servomin: u16, servomax: u16) -> u16 {
    let input_min = 0;
    let input_max = 180;
    servomin + (angle - input_min) * (servomax - servomin) / (input_max - input_min)
}

#[cfg(feature = "runtime")]
fn move_servo(pca: &mut Pca9685, idx: u8, angle: u16) -> rppal::i2c::Result<()> {
    let len = map_angle_to_pulse(angle, SERVO_MIN, SERVO_MAX);
    pca.set_pwm(idx, 0, len)?;

    Ok(())
}
