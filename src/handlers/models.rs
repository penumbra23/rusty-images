use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct ImageStats {
    size: usize,
    width: u32,
    height: u32,
    format: String,
}

impl ImageStats {
    pub fn new(width: u32, height: u32, size: usize, format: String) -> ImageStats {
        ImageStats { size, width, height, format }
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