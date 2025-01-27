use http_body_util::Full;
use hyper::{
    body::{self, Bytes},
    service::Service,
    Method, Request, Response, StatusCode,
};
use std::{collections::HashMap, fs::File, future::Future, io::Read, pin::Pin, sync::Arc};
use tokio::sync::{mpsc::Sender, Mutex};
use url::Url;

use crate::msg::PuppyMsg;

/// The service for holding context on all servos
#[derive(Clone, Debug)]
pub struct PuppyService {
    controller_send: Sender<PuppyMsg>,
    servos: Arc<Mutex<HashMap<u8, Servo>>>,
}

impl PuppyService {
    /// Creates a new puppy service with respect to a controller sender channel
    pub fn with_send(controller_send: Sender<PuppyMsg>) -> Self {
        Self {
            controller_send,
            servos: Arc::new(Mutex::new(HashMap::default())),
        }
    }
    /// Register a servo with a name
    pub async fn register(&mut self, name: &'static str, index: u8) {
        self.servos.lock().await.insert(
            index,
            Servo {
                name,
                ..Default::default()
            },
        );
    }

    /// Asign a new angle to a servo
    pub async fn assign_angle(&self, servo: u8, angle: u16) {
        if let Some(servo) = self.servos.lock().await.get_mut(&servo) {
            servo.set_angle(angle);
        }
    }

    /// Sets the current angles on all servos to zero
    pub async fn set_zero_offsets(&self) {
        self.servos
            .lock()
            .await
            .values_mut()
            .for_each(|servo| servo.set_zero_offset());
    }
}

/// Internal context for servo motor
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct Servo {
    /// Motor name
    name: &'static str,
    /// Current servo angle
    angle: u16,
    /// The offset to zero the servo
    zero_offset: u16,
}

impl Servo {
    /// Sets a new angle
    pub fn set_angle(&mut self, angle: u16) {
        self.angle = angle
    }

    /// Assigns the current angle to be the zero offset
    pub fn set_zero_offset(&mut self) {
        self.zero_offset = self.angle
    }
}

impl Service<Request<body::Incoming>> for PuppyService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<body::Incoming>) -> Self::Future {
        let service = self.clone();
        Box::pin(async move {
            let response = Response::builder();

            match (req.method(), req.uri().path()) {
                (&Method::GET, "/") => {
                    let mut buf = vec![];
                    let mut page = File::open("frontend/index.html").expect("Failed to find file");
                    page.read_to_end(&mut buf)
                        .expect("Failed to read to buffer");
                    response
                        .status(StatusCode::OK)
                        .body(Full::new(Bytes::copy_from_slice(&buf)))
                }

                (&Method::GET, "/get-servos") => {
                    todo!("Get all servos with their current angles and names")
                }

                (&Method::GET, "/move") => {
                    let uri = req.uri().to_string();
                    let request_url = Url::parse(&format!("https://dumbfix.com/{}", uri)).unwrap();

                    let query = request_url
                        .query_pairs()
                        .map(|(key, val)| (key.to_string(), val.to_string()))
                        .collect::<HashMap<_, _>>();

                    let servo = query["servo"].parse().expect("Parse to u8");
                    let angle = query["angle"].parse().expect("Parse to u16");

                    service
                        .controller_send
                        .send(PuppyMsg::MoveServe(servo, angle))
                        .await
                        .expect("Send angle move to dog");

                    service.assign_angle(servo, angle).await;

                    response
                        .status(StatusCode::OK)
                        .body(Full::new(Bytes::copy_from_slice(b"Yippee!")))
                }

                (&Method::POST, "/set-zeroes") => {
                    service.set_zero_offsets().await;

                    response
                        .status(StatusCode::OK)
                        .body(Full::new(Bytes::copy_from_slice(b"Yippee!")))
                }

                (&Method::GET, "/favicon.ico") => {
                    let mut buf = vec![];
                    let mut page = File::open("frontend/favicon.ico").expect("Failed to find file");
                    page.read_to_end(&mut buf)
                        .expect("Failed to read to buffer");
                    response
                        .status(StatusCode::OK)
                        .body(Full::new(Bytes::copy_from_slice(&buf)))
                }

                _ => unimplemented!(),
            }
        })
    }
}
