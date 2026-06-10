//! Vault: per-owner storage, issuer management, credential storage.

mod credential;
mod issuer;

pub use credential::{charge_fee, charge_fee_quote_only, revoke_vc, store_vc, transfer_fee};
pub use issuer::{authorize_issuer, authorize_issuers, is_authorized, revoke_issuer};
