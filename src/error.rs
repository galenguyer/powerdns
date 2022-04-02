use serde::Deserialize;

/// Returned when the server encounters an error, either in client input or
/// internally
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde_with::skip_serializing_none]
pub struct Error {
    /// A human readable error message
    pub error: String,
    /// Optional array of multiple errors encountered during processing
    pub errors: Option<Vec<String>>,
}
