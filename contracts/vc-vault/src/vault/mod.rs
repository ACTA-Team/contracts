//! Vault: per-owner storage, issuer management, credential storage.

mod credential;
mod issuer;

pub use credential::{charge_fee, revoke_vc, store_vc};
pub use issuer::{authorize_issuer, authorize_issuers, is_authorized, revoke_issuer};
