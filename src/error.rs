use std::fmt::{Debug, Display, Formatter};
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

/// Returned when the server encounters an error, either in client input or
/// internally
#[derive(Error, Debug)]
#[serde_with::skip_serializing_none]
pub enum Error {
    #[error("powerdns returned error in response: {0:?}")]
    PowerDNS(#[from] PowerDNSResponseError),

    #[error("error while performing request: {}", 0)]
    RequestError(#[from] reqwest::Error),

    #[error("received unexpected status code: {}", 0)]
    UnexpectedStatusCode(StatusCode),

    #[error("deserialization error: {0}")]
    DeserializeError(#[from] serde_json::Error),

    #[error("other error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>)
}


/// Represents PowerDNS response error
#[derive(Default, PartialEq, Debug, Clone, Deserialize)]
pub struct PowerDNSResponseError {
    /// A human readable error message
    pub error: String,
    /// Optional array of multiple errors encountered during processing
    pub errors: Option<Vec<String>>,
}

impl Display for PowerDNSResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("error occurred: {}", self.error))
    }
}

impl std::error::Error for PowerDNSResponseError {

}