mod api;
mod error;
mod runtime;
mod types;

pub use api::VaultSigner;
pub use error::SignerError;
pub use types::{FfiProposal, FfiProposalSummary, FfiVaultConfig};

uniffi::setup_scaffolding!();
