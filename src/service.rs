use http_body_util::Full;
use hyper::{
    body::{self, Bytes},
    service::Service,
    Method, Request, Response, StatusCode,
};
use std::{collections::HashMap, fs::File, future::Future, io::Read, pin::Pin};
use tokio::sync::mpsc::Sender;

use crate::msg::PuppyMsg;

/// The service for holding context on all servos
#[derive(Clone, Debug)]
pub struct PuppyService {
    controller_send: Sender<PuppyMsg>,
    servos: HashMap<&'static str, Servo>,
}

impl PuppyService {
    /// Creates a new puppy service with respect to a controller sender channel
    pub fn with_send(controller_send: Sender<PuppyMsg>) -> Self {
        Self {
            controller_send,
            servos: HashMap::default(),
        }
    }
    /// Register a servo with a name
    pub fn register(&mut self, name: &'static str, index: usize) {
        self.servos.insert(
            name,
            Servo {
                index,
                ..Default::default()
            },
        );
    }
}

/// Internal context for servo motor
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct Servo {
    /// Index in the MotorController
    index: usize,
    /// Current servo angle
    angle: f32,
    /// The offset to zero the servo
    zero_offset: f32,
}

impl Service<Request<body::Incoming>> for PuppyService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<body::Incoming>) -> Self::Future {
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

                (&Method::POST, "/move") => {
                    todo!("Move servo to new angle");
                }

                (&Method::POST, "/zero-align-servo") => {
                    todo!("Set a servo's zero offset")
                }

                _ => unimplemented!(),
            }
        })
    }
}
