# references/accounts.md: fuzz accounts, PDA seeds, reading state

Trident generates `AccountAddresses` (in `fuzz_accounts.rs`) with one `AddressStorage` per account your instructions touch. You allocate concrete addresses from it inside flows.

## Keypairs and PDAs

```rust
// fresh keypair signer
let authority = self.fuzz_accounts.authority.insert(&mut self.trident, None);

// PDA: the seeds MUST match the program's #[account(seeds = [...])]
let vault = self.fuzz_accounts.vault.insert(
    &mut self.trident,
    Some(PdaSeeds {
        seeds: &[b"vault".as_ref(), authority.as_ref()],
        program_id: program_id(),
    }),
);
```

The seeds passed to `insert` have to match the program byte-for-byte. The vault program declares `seeds = [b"vault", authority.key().as_ref()]`, so the fuzz seeds are `&[b"vault".as_ref(), authority.as_ref()]`. Note `b"vault".as_ref()`: the array literal needs every element to be `&[u8]`, and a bare `b"vault"` is `&[u8; 5]`, so `.as_ref()` coerces it.

A seed mismatch produces a different address, and then every transaction fails. The symptom is a metrics table where the instruction shows 0 successes; if you see that, check the seeds first.

## Reading typed state

```rust
let v = self.trident.get_account_with_type::<Vault>(&vault, 8);  // -> Option<Vault>
```

`get_account_with_type::<T>(key, discriminator_size)` reads the account data and deserializes `T` after skipping `discriminator_size` bytes (`8` for Anchor's discriminator). It returns `None` when the account does not exist or is smaller than the discriminator, so handle the `Option` during setup rather than unwrapping blindly: the account does not exist until `initialize` has run.

The state type itself (`Vault` here) is generated into `types.rs` deriving `BorshDeserialize`, so you can deserialize it directly. Do not edit `types.rs` by hand; it is regenerated from the IDL.
