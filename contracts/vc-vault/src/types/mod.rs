//! Shared data types. No storage dependencies; used by storage, vault, issuance.

mod fee_quote;
mod vc_status;
mod verifiable_credential;

pub use fee_quote::FeeQuote;
pub use vc_status::VCStatus;
pub use verifiable_credential::VerifiableCredential;
