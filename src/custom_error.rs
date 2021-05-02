use std::{error::Error, fmt::{Display}};

#[derive(Debug)]
pub enum CustomError {
    ExtractionError(String),
    ConfigurationError(String),
    CommandError(String),
}

impl Error for CustomError {}

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            CustomError::ExtractionError(s) => {
                f.write_str(&*s)
            },
            CustomError::ConfigurationError(s) => {
                f.write_str(&*s)
            },
            CustomError::CommandError(s) => {
                f.write_str(&*s)
            },
        }
    }
}
