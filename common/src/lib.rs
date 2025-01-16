use std::{
    fmt,
    io::{Read, Write},
    net::TcpStream,
};

pub struct Position {
    pub x: usize,
    pub y: usize,
}
impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
    pub fn new_from_pos(pos: &Position) -> Self {
        Self { x: pos.x, y: pos.y }
    }
}

pub fn send_ok(stream: &Option<TcpStream>) {
    stream
        .as_ref()
        .unwrap()
        .write_all(String::from("ok").as_bytes())
        .expect("Eroare la send ok(write)");
}

pub fn read_ok(stream: &Option<TcpStream>) {
    let mut buffer = [0; 1024];
    let _n = stream
        .as_ref()
        .unwrap()
        .read(&mut buffer)
        .expect("Eroare la read");
}

pub fn convert_to_i32(s: &str) -> i32 {
    match s.parse::<i32>() {
        Ok(num) => num,
        Err(e) => {
            println!("Eroare la parsare: {}", e);
            -1
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    ConnectionError(String),
    CloneError(String),
    StreamUnavailable(String),
    WriteError(String),
    ReadError(String),
    InvalidMove(String),
    ServerError(String),
    UnexpectedResponse(String),
    IOError(std::io::Error),
    UTF8Error(String),
    GraphicsError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            AppError::CloneError(msg) => write!(f, "Clone error: {}", msg),
            AppError::StreamUnavailable(msg) => write!(f, "Stream is unavailable {}", msg),
            AppError::WriteError(msg) => write!(f, "Write error: {}", msg),
            AppError::ReadError(msg) => write!(f, "Read error: {}", msg),
            AppError::InvalidMove(msg) => write!(f, "Invalid move: {}", msg),
            AppError::ServerError(msg) => write!(f, "Server error: {}", msg),
            AppError::UnexpectedResponse(msg) => write!(f, "Unexpected response: {}", msg),
            AppError::IOError(err) => write!(f, "IO error: {}", err),
            AppError::UTF8Error(msg) => write!(f, "UTF-8 error: {}", msg),
            AppError::GraphicsError(msg) => write!(f, "Egui error: {}", msg),
        }
    }
}
impl AppError {
    pub fn log(self) -> Self {
        eprintln!("Error: {}", self);
        self
    }
}
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IOError(err)
    }
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        AppError::UTF8Error(err.to_string())
    }
}
