//! Vault: per-owner storage, issuer management, credential storage.

mod credential;

pub use credential::{charge_fee, charge_fee_quote_only, revoke_vc, store_vc, transfer_fee};
