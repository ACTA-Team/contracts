//! Vault: per-owner storage, issuer management, credential storage.

mod credential;
mod issuer;

pub use credential::{push_vc, revoke_vc, store_vc, store_vc_with_fee};
pub use issuer::{authorize_issuer, authorize_issuers, is_authorized, revoke_issuer};
