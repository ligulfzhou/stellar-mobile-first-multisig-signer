use thiserror::Error;

#[derive(Debug, Error, uniffi::Error)]
pub enum SignerError {
    #[error("{msg}")]
    Generic { msg: String },
}

impl SignerError {
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic { msg: msg.into() }
    }
}

impl From<anyhow::Error> for SignerError {
    fn from(err: anyhow::Error) -> Self {
        Self::generic(err.to_string())
    }
}
