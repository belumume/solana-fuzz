use trident_fuzz::fuzzing::*;

/// Storage for all account addresses used in fuzz testing.
///
/// This struct serves as a centralized repository for account addresses,
/// enabling their reuse across different instruction flows and test scenarios.
///
/// Docs: https://ackee.xyz/trident/docs/latest/trident-api-macro/trident-types/fuzz-accounts/
#[derive(Default)]
pub struct AccountAddresses {
    pub vault: AddressStorage,

    pub authority: AddressStorage,

    // Trident emits one storage slot per IDL account, including system_program.
    // The Initialize builder fills the system program internally, so this slot
    // needs no manual insert. Kept to match the generated layout.
    pub system_program: AddressStorage,
}
