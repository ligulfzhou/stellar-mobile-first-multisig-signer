use {
    crate::{types::*, VaultClient},
    anyhow::{anyhow, Result},
    futures::future::try_join_all,
    soroban_client::xdr::ScVal,
    stellar_core::scval::{
        map_get_field, scval_to_address_string, scval_to_bool, scval_to_i128, scval_to_string, scval_to_u32,
        scval_to_u64, scval_to_vec, vec_get_field,
    },
};

pub struct VaultReader<'a> {
    pub client: &'a VaultClient,
}

impl VaultReader<'_> {
    pub async fn get_config(&self) -> Result<VaultConfig> {
        let val = self
            .client
            .rpc
            .simulate_contract_call(&self.client.vault, "get_config", vec![])
            .await?;
        parse_vault_config(&val)
    }

    pub async fn get_signers(&self) -> Result<Vec<String>> {
        let val = self
            .client
            .rpc
            .simulate_contract_call(&self.client.vault, "get_signers", vec![])
            .await?;

        let items = scval_to_vec(&val)?;
        items.iter().map(scval_to_address_string).collect()
    }

    pub async fn get_proposal(&self, proposal_id: u64) -> Result<ProposalCore> {
        let args = vec![stellar_core::u64_to_scval(proposal_id)];
        let val = self
            .client
            .rpc
            .simulate_contract_call(&self.client.vault, "get_proposal", args)
            .await?;
        parse_proposal_core(&val)
    }

    /// List all proposals (1..=proposal_count) with on-chain status.
    pub async fn list_proposals(&self) -> Result<Vec<ProposalSummary>> {
        let config = self.get_config().await?;
        if config.proposal_count == 0 {
            return Ok(vec![]);
        }
        let cores = try_join_all((1..=config.proposal_count).map(|id| self.get_proposal(id))).await?;
        Ok(cores
            .into_iter()
            .enumerate()
            .map(|(i, core)| ProposalSummary {
                id: (i as u64) + 1,
                proposal_type: core.proposal_type,
                approval_count: core.approval_count,
                rejection_count: core.rejection_count,
                status: core.status_label().to_string(),
            })
            .collect())
    }

    /// Pending proposals only (not executed, not rejected).
    pub async fn list_pending_proposals(&self) -> Result<Vec<ProposalSummary>> {
        Ok(self
            .list_proposals()
            .await?
            .into_iter()
            .filter(|p| p.status == "pending")
            .collect())
    }
}

fn parse_vault_config(val: &ScVal) -> Result<VaultConfig> {
    let field =
        |name: &str, idx: usize| -> Result<ScVal> { map_get_field(val, name).or_else(|_| vec_get_field(val, idx)) };

    let name = scval_to_string(&field("name", 0)?)?;
    Ok(VaultConfig {
        name,
        threshold: scval_to_u32(&field("threshold", 1)?)?,
        signer_count: scval_to_u32(&field("signer_count", 2)?)?,
        proposal_count: scval_to_u64(&field("proposal_count", 3)?)?,
        lock_count: scval_to_u64(&field("lock_count", 4)?)?,
        fee_amount: scval_to_i128(&field("fee_amount", 5)?)?,
    })
}

fn parse_proposal_core(val: &ScVal) -> Result<ProposalCore> {
    let field =
        |name: &str, idx: usize| -> Result<ScVal> { map_get_field(val, name).or_else(|_| vec_get_field(val, idx)) };

    let proposal_type_raw = scval_to_u32(&field("proposal_type", 0)?)?;
    let proposal_type = ProposalType::from_u32(proposal_type_raw)
        .ok_or_else(|| anyhow!("unknown proposal_type: {}", proposal_type_raw))?;

    Ok(ProposalCore {
        proposal_type,
        approval_count: scval_to_u32(&field("approval_count", 1)?)?,
        rejection_count: scval_to_u32(&field("rejection_count", 2)?)?,
        is_executed: scval_to_bool(&field("is_executed", 3)?)?,
        is_rejected: scval_to_bool(&field("is_rejected", 4)?)?,
    })
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        soroban_client::xdr::{Int128Parts, ScSymbol, ScVal},
    };

    #[test]
    fn parses_vault_config_from_vec() {
        let val = ScVal::Vec(Some(
            vec![
                ScVal::Symbol(ScSymbol::try_from("team_treasury").unwrap()),
                ScVal::U32(2),
                ScVal::U32(3),
                ScVal::U64(5),
                ScVal::U64(0),
                ScVal::I128(Int128Parts { hi: 0, lo: 1_000_000 }),
            ]
            .try_into()
            .unwrap(),
        ));

        let cfg = parse_vault_config(&val).unwrap();
        assert_eq!(cfg.name, "team_treasury");
        assert_eq!(cfg.threshold, 2);
        assert_eq!(cfg.proposal_count, 5);
    }
}
