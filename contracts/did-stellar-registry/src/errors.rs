//! Contract error codes.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RegistryError {
    /// `register` called with a `did_id` that already has a record.
    DidAlreadyExists = 1,
    /// `update`/`transfer_controller`/`deactivate` called for an unknown DID.
    DidNotFound = 2,
    /// `expected_version` does not match the current on-chain `version`.
    VersionMismatch = 3,
    /// Mutation attempted on a record where `deactivated == true`.
    DidDeactivated = 4,
    /// `authentication` count is outside [MIN_KEY_COUNT_AUTH, MAX_KEY_COUNT_AUTH].
    InvalidAuthKeyCount = 5,
    /// `assertion_method` count exceeds MAX_KEY_COUNT_ASSERT.
    InvalidAssertionKeyCount = 6,
    /// `key_agreement` count exceeds MAX_KEY_COUNT_AGREEMENT.
    InvalidKeyAgreementCount = 7,
    /// `services` count exceeds MAX_SERVICE_COUNT.
    InvalidServiceCount = 8,
    /// Same `public_key_multibase` appears twice in a single relationship.
    DuplicateKey = 9,
    /// `public_key_multibase` exceeds MAX_KEY_MULTIBASE_LEN.
    KeyTooLong = 10,
    /// `public_key_multibase` is empty.
    KeyEmpty = 11,
    /// `service.service_type` exceeds MAX_SERVICE_TYPE_LEN.
    ServiceTypeTooLong = 12,
    /// `service.id_suffix` exceeds MAX_SERVICE_ID_LEN.
    ServiceIdTooLong = 13,
    /// `service.id_suffix` does not match `^[a-z0-9-]+$`.
    ServiceIdInvalidFormat = 14,
    /// `service.service_endpoint` is not an `https://` URL or exceeds MAX_URL_LEN.
    ServiceEndpointInvalid = 15,
    /// `metadata_uri` is not an `https://` URL or exceeds MAX_URL_LEN.
    MetadataUriInvalid = 16,
    /// `accept_admin` called when no admin has been proposed (or the
    /// proposal expired from temporary storage).
    NoProposedAdmin = 17,
    /// `service.service_type` is empty.
    ServiceTypeEmpty = 18,
    /// `update`/`transfer_controller`/`deactivate` called on a DID whose
    /// `version` has reached `u32::MAX`. Extremely unlikely in practice but
    /// must be handled to prevent arithmetic overflow.
    VersionOverflow = 19,
    /// `metadata_hash` is `Some` but `metadata_uri` is `None`. An integrity
    /// hash without a corresponding URI is orphaned and meaningless.
    MetadataInconsistent = 20,
    /// Two services share the same `id_suffix`, which would produce duplicate
    /// `{did}#service-{id_suffix}` fragments in the resolved DID Document.
    DuplicateServiceId = 21,
}
