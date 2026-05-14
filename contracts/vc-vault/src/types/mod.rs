//! Shared data types. No storage dependencies; used by storage, vault, issuance.

mod vc_status;
mod verifiable_credential;

pub use vc_status::VCStatus;
pub use verifiable_credential::VerifiableCredential;
