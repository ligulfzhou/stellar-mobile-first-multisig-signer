use vault_client::{ProposalCore, ProposalSummary, ProposalType, VaultConfig};

#[derive(uniffi::Record)]
pub struct FfiVaultConfig {
    pub name: String,
    pub threshold: u32,
    pub signer_count: u32,
    pub proposal_count: u64,
    pub lock_count: u64,
    /// Stroops, serialized as string (i128 not supported in UniFFI).
    pub fee_amount: String,
}

#[derive(uniffi::Record)]
pub struct FfiProposalSummary {
    pub id: u64,
    pub proposal_type: String,
    pub approval_count: u32,
    pub rejection_count: u32,
    pub status: String,
}

#[derive(uniffi::Record)]
pub struct FfiProposal {
    pub proposal_type: String,
    pub approval_count: u32,
    pub rejection_count: u32,
    pub status: String,
}

impl From<VaultConfig> for FfiVaultConfig {
    fn from(value: VaultConfig) -> Self {
        Self {
            name: value.name,
            threshold: value.threshold,
            signer_count: value.signer_count,
            proposal_count: value.proposal_count,
            lock_count: value.lock_count,
            fee_amount: value.fee_amount.to_string(),
        }
    }
}

impl From<ProposalSummary> for FfiProposalSummary {
    fn from(value: ProposalSummary) -> Self {
        Self {
            id: value.id,
            proposal_type: proposal_type_label(value.proposal_type).to_string(),
            approval_count: value.approval_count,
            rejection_count: value.rejection_count,
            status: value.status,
        }
    }
}

impl From<ProposalCore> for FfiProposal {
    fn from(value: ProposalCore) -> Self {
        Self {
            proposal_type: proposal_type_label(value.proposal_type).to_string(),
            approval_count: value.approval_count,
            rejection_count: value.rejection_count,
            status: value.status_label().to_string(),
        }
    }
}

fn proposal_type_label(kind: ProposalType) -> &'static str {
    match kind {
        ProposalType::Transfer => "transfer",
        ProposalType::TimeLock => "timelock",
        ProposalType::VestingLock => "vesting",
        ProposalType::AddSigner => "add_signer",
        ProposalType::RemoveSigner => "remove_signer",
        ProposalType::SetRole => "set_role",
        ProposalType::SetThreshold => "set_threshold",
    }
}
