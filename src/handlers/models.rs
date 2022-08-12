use std::f32::consts::E;

use serde::{Serialize, de::Error};

#[derive(Clone, Serialize)]
pub struct ImageStats {
    size: u64
}

impl ImageStats {
    pub fn new(size: u64) -> ImageStats {
        ImageStats { size }
    }
}

#[derive(Serialize)]
pub struct ErrorMessage {
    message: String,
}

impl ErrorMessage {
    pub fn new(msg: &str) -> ErrorMessage {
        ErrorMessage { message: msg.to_owned() }
    }
}