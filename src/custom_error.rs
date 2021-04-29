use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct CustomError {
    description: String,
}

impl Error for CustomError {}

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&*self.description)
    }
}

impl From<&str> for CustomError {
    fn from(s: &str) -> Self {
        CustomError {
            description: s.into()
        }
    }
}
