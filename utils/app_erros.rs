#[derive(Debug)]
pub enum AppError {
    UnknownErrors(string),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::UnknownErrors(err) => eprintl!(f, "Error: {}", err),
        }
    }
}
