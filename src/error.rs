//! Error types

use std::{error::Error as StdError, fmt, io};

#[derive(Debug)]
pub enum Error {
    /// Reqwest http error
    HttpError(reqwest::Error),
    /// Feed URl parser error
    CouldNotParseRSSFromUrl(String),
    /// IO Error
    IO(io::Error),
    /// Error while sending email
    MailError(lettre::transport::smtp::Error),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::HttpError(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<lettre::transport::smtp::Error> for Error {
    fn from(e: lettre::transport::smtp::Error) -> Self {
        Self::MailError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::HttpError(ref e) => write!(f, "{}", e),
            Self::CouldNotParseRSSFromUrl(ref e) => write!(f, "{}", e),
            Self::IO(ref e) => write!(f, "{}", e),
            Self::MailError(ref e) => write!(f, "{}", e),
        }
    }
}

impl StdError for Error {}
