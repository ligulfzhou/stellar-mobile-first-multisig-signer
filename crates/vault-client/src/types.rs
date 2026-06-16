use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VaultConfig {
    pub name: String,
    pub threshold: u32,
    pub signer_count: u32,
    pub proposal_count: u64,
    pub lock_count: u64,
    pub fee_amount: i128,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u32)]
pub enum ProposalType {
    Transfer = 0,
    TimeLock = 1,
    VestingLock = 2,
    AddSigner = 3,
    RemoveSigner = 4,
    SetRole = 5,
    SetThreshold = 6,
}

impl ProposalType {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::Transfer),
            1 => Some(Self::TimeLock),
            2 => Some(Self::VestingLock),
            3 => Some(Self::AddSigner),
            4 => Some(Self::RemoveSigner),
            5 => Some(Self::SetRole),
            6 => Some(Self::SetThreshold),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProposalSummary {
    pub id: u64,
    pub proposal_type: ProposalType,
    pub approval_count: u32,
    pub rejection_count: u32,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProposalCore {
    pub proposal_type: ProposalType,
    pub approval_count: u32,
    pub rejection_count: u32,
    pub is_executed: bool,
    pub is_rejected: bool,
}

impl ProposalCore {
    pub fn status_label(&self) -> &'static str {
        if self.is_executed {
            "executed"
        } else if self.is_rejected {
            "rejected"
        } else {
            "pending"
        }
    }
}
