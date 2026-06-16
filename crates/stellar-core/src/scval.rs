use {
    anyhow::{anyhow, Result},
    soroban_client::xdr::{self, ScVal},
    stellar_strkey::{ed25519::PublicKey, Contract},
};

pub fn address_to_scval(address: &str) -> Result<ScVal> {
    if address.starts_with('G') {
        let pk = PublicKey::from_string(address).map_err(|e| anyhow!("invalid G address: {}", e))?;
        let account_id = xdr::AccountId(xdr::PublicKey::PublicKeyTypeEd25519(xdr::Uint256(pk.0)));
        Ok(ScVal::Address(xdr::ScAddress::Account(account_id)))
    } else if address.starts_with('C') {
        let contract = Contract::from_string(address).map_err(|e| anyhow!("invalid C address: {}", e))?;
        Ok(ScVal::Address(xdr::ScAddress::Contract(xdr::ContractId(xdr::Hash(
            contract.0,
        )))))
    } else {
        Err(anyhow!("address must start with G or C, got {}", address))
    }
}

pub fn u32_to_scval(value: u32) -> ScVal {
    ScVal::U32(value)
}

pub fn u64_to_scval(value: u64) -> ScVal {
    ScVal::U64(value)
}

pub fn i128_to_scval(value: i128) -> ScVal {
    ScVal::I128(xdr::Int128Parts {
        hi: (value >> 64) as i64,
        lo: value as u64,
    })
}

pub fn bool_to_scval(value: bool) -> ScVal {
    ScVal::Bool(value)
}

pub fn symbol_to_scval(value: &str) -> Result<ScVal> {
    let sym: xdr::ScSymbol = value.try_into().map_err(|_| anyhow!("invalid symbol: {}", value))?;
    Ok(ScVal::Symbol(sym))
}

pub fn scval_to_u32(val: &ScVal) -> Result<u32> {
    match val {
        ScVal::U32(v) => Ok(*v),
        _ => Err(anyhow!("expected U32, got {:?}", val)),
    }
}

pub fn scval_to_u64(val: &ScVal) -> Result<u64> {
    match val {
        ScVal::U64(v) => Ok(*v),
        _ => Err(anyhow!("expected U64, got {:?}", val)),
    }
}

pub fn scval_to_i128(val: &ScVal) -> Result<i128> {
    match val {
        ScVal::I128(parts) => Ok(((parts.hi as i128) << 64) | (parts.lo as u64 as i128)),
        _ => Err(anyhow!("expected I128, got {:?}", val)),
    }
}

pub fn scval_to_bool(val: &ScVal) -> Result<bool> {
    match val {
        ScVal::Bool(v) => Ok(*v),
        _ => Err(anyhow!("expected Bool, got {:?}", val)),
    }
}

pub fn scval_to_string(val: &ScVal) -> Result<String> {
    match val {
        ScVal::String(s) => Ok(s.to_string()),
        ScVal::Symbol(s) => Ok(s.to_string()),
        _ => Err(anyhow!("expected String/Symbol, got {:?}", val)),
    }
}

pub fn scval_to_address_string(val: &ScVal) -> Result<String> {
    match val {
        ScVal::Address(xdr::ScAddress::Contract(xdr::ContractId(hash))) => Ok(format!("{}", Contract(hash.0))),
        ScVal::Address(xdr::ScAddress::Account(account_id)) => match account_id {
            xdr::AccountId(xdr::PublicKey::PublicKeyTypeEd25519(key)) => Ok(format!("{}", PublicKey(key.0))),
        },
        _ => Err(anyhow!("expected Address, got {:?}", val)),
    }
}

pub fn scval_to_vec(val: &ScVal) -> Result<Vec<ScVal>> {
    match val {
        ScVal::Vec(Some(v)) => Ok(v.to_vec()),
        ScVal::Vec(None) => Ok(vec![]),
        _ => Err(anyhow!("expected Vec, got {:?}", val)),
    }
}

pub fn map_get_field(val: &ScVal, field: &str) -> Result<ScVal> {
    match val {
        ScVal::Map(Some(map)) => {
            for entry in map.iter() {
                if scval_to_string(&entry.key).map(|s| s == field).unwrap_or(false) {
                    return Ok(entry.val.clone());
                }
            }
            Err(anyhow!("field {} not found in map", field))
        }
        _ => Err(anyhow!("expected Map for field lookup, got {:?}", val)),
    }
}

pub fn vec_get_field(val: &ScVal, index: usize) -> Result<ScVal> {
    let items = scval_to_vec(val)?;
    items
        .into_iter()
        .nth(index)
        .ok_or_else(|| anyhow!("index {} out of range", index))
}
