//! Error types

use std::{error::Error as StdError, fmt, io};

#[derive(Debug)]
pub enum Error {
    ErrorSendingMail(String),
    /// Reqwest http error
    Http(String),
    /// Feed URl parser error
    CouldNotParseRSSFromUrl(String),
    /// IO Error
    IO(io::Error),
    /// Error while sending email
    MailContentError(String),
    SmtpError(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<lettre::error::Error> for Error {
    fn from(e: lettre::error::Error) -> Self {
        Self::MailContentError(e.to_string())
    }
}

impl From<lettre::transport::smtp::Error> for Error {
    fn from(e: lettre::transport::smtp::Error) -> Self {
        Self::SmtpError(e.to_string())
    }
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Http(ref e) => write!(f, "{}", e),
            Self::CouldNotParseRSSFromUrl(ref e) => write!(f, "{}", e),
            Self::IO(ref e) => write!(f, "{}", e),
            Self::MailContentError(ref e) => write!(f, "{}", e),
            Self::ErrorSendingMail(ref e) => write!(f, "{}", e),
            Self::SmtpError(ref e) => write!(f, "{}", e),
        }
    }
}
