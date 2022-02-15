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
    MailTransport(String),
    /// Invalid email address
    AddressError(String),
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

impl From<lettre::transport::smtp::Error> for Error {
    fn from(e: lettre::transport::smtp::Error) -> Self {
        Self::MailTransport(e.to_string())
    }
}

impl From<lettre::address::AddressError> for Error {
    fn from(e: lettre::address::AddressError) -> Self {
        Self::AddressError(e.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Http(ref e) => write!(f, "{}", e),
            Self::CouldNotParseRSSFromUrl(ref e) => write!(f, "{}", e),
            Self::IO(ref e) => write!(f, "{}", e),
            Self::MailTransport(ref e) => write!(f, "{}", e),
            Self::AddressError(ref e) => write!(f, "{}", e),
            Self::ErrorSendingMail(ref e) => write!(f, "{}", e),
        }
    }
}

impl StdError for Error {}
