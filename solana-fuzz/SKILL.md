---
name: solana-fuzz
description: >
  Use when fuzzing a Solana program or writing a Trident fuzz test: property and
  invariant testing of a program's state transitions, finding runtime logic bugs
  that happy-path and example-based tests (LiteSVM, Mollusk, Surfpool) miss,
  hardening an Anchor or native program before a mainnet deploy, or adding
  regression fuzzing across program versions. Trident is the Solana Foundation
  supported fuzzer by Ackee Blockchain. Dynamic, dev-loop property testing, not a
  static security auditor.
license: MIT
compatibility: "claude-code, codex"
metadata:
  version: "2026-06"
  trident-version: "0.12.0"
  trident-source: "https://github.com/Ackee-Blockchain/trident"
  positioning: "Complements the kit's example-based testing (LiteSVM/Mollusk/Surfpool); distinct from static security-audit skills."
---

# solana-fuzz: Trident fuzzing and invariants for Solana programs

Turn a Solana program into a property-tested one. This skill writes Trident fuzz
tests: it scaffolds the test layout, generates the flow methods that drive your
instructions with random inputs, and writes invariant methods that assert your
program's state stays correct after every fuzzed transaction sequence. Trident
(Ackee Blockchain, Solana Foundation supported) runs many transactions per second
against the TridentSVM client and catches the runtime logic errors that happy-path
example tests never reach.

> Verify before you author. Trident's API changes between minor versions. This
> skill pins to v0.12.0 (see `metadata.trident-version`), the latest stable release;
> 0.13 is still pre-release (RC) as of this skill's date, so stable is the
> reproducible choice. Before generating code, confirm the user's installed version
> and re-read the relevant reference file rather than authoring from memory. If the
> version differs, check the live source at the URL in `metadata.trident-source`.

## When to use this skill

- The user wants to fuzz a Solana program, or write a Trident fuzz test.
- The user wants property/invariant checks over a program's state transitions.
- The user wants to find runtime bugs that example-based tests miss, or harden a
  program before a mainnet deploy.
- The user wants regression fuzzing to catch bugs introduced between versions.

## What this skill is NOT

- Not a static security auditor or vulnerability scanner. Route code-audit and
  exploit-class detection to a dedicated audit skill. This skill is dynamic,
  dev-loop property testing.
- Not a replacement for example-based tests. It complements LiteSVM/Mollusk/Surfpool.
- Not a transaction-landing, signing, or RPC tool. It runs against TridentSVM, not
  a live cluster.

## The Trident model (v0.12.0)

Anchored to the repo source, not memory:

- Install the CLI: `cargo install trident-cli --locked`. Run a suite from the `trident-tests/`
  directory: `trident fuzz run fuzz_0 --with-exit-code`. The `--with-exit-code` flag is
  required: without it a failing invariant is silently swallowed in parallel mode and the
  run still exits 0 (see `references/invariants.md`).
- A fuzz test lives in `trident-tests/fuzz_0/` and is made of:
  `test_fuzz.rs`, `fuzz_accounts.rs`, `types.rs`, and `Trident.toml`.
- The test is a struct deriving the framework methods and an impl annotated as a
  flow executor:

  ```rust
  use trident_fuzz::fuzzing::*;

  #[derive(FuzzTestMethods)]
  struct FuzzTest {
      trident: Trident,
      fuzz_accounts: AccountAddresses,
  }

  #[flow_executor]
  impl FuzzTest {
      fn new() -> Self { Self { trident: Trident::default(), fuzz_accounts: AccountAddresses::default() } }

      #[init] fn start(&mut self) { /* per-iteration setup */ }
      #[flow] fn flow1(&mut self) { /* randomly selected each iteration */ }
      #[end]  fn end(&mut self)   { /* per-iteration cleanup */ }
  }

  fn main() { FuzzTest::fuzz(1000, 100); }
  ```

- Drive an instruction: build it, then
  `self.trident.process_transaction(&[ix], Some("Label"))`, which returns a result
  with `.is_success()` and `.get_transaction_timestamp()`. Gate invariant checks on
  `.is_success()`: a rejected transaction is the expected-failure path, not a violation.
- Random inputs: `self.trident.random_from_range(0..u8::MAX)`. Fund a signer:
  `self.trident.airdrop(&addr, 10 * LAMPORTS_PER_SOL)`. Advance the clock:
  `self.trident.forward_in_time(n)`.
- Insert accounts in `fuzz_accounts`: `self.fuzz_accounts.author.insert(&mut self.trident, None)`
  for a keypair, or with `Some(PdaSeeds { seeds: &[b"seed"], program_id })` for a PDA.
- Read typed state: `self.trident.get_account_with_type::<MyAccount>(&addr, 8)`
  (the `8` is the discriminator byte offset for Anchor accounts).

## Invariants (the core value)

The pattern: capture state before, execute, capture state after, assert the
expected change in a dedicated invariant method. Handle expected failures
explicitly instead of swallowing them.

```rust
let before = match self.trident.get_account_with_type::<MyAccount>(&addr, 8) {
    Some(v) => v,            // 8 = Anchor discriminator; None until the account is initialized
    None => return,
};
let res = self.trident.process_transaction(&[ix], Some("op"));
if res.is_success() {
    if let Some(after) = self.trident.get_account_with_type::<MyAccount>(&addr, 8) {
        assert!(/* domain property over before/after */, "describe the violation");
    }
}
```

Gate the assert on `is_success()` so a correctly-rejected transaction is not counted
as a violation. The full worked invariant method, the expected-failure handling, and
the mandatory `--with-exit-code` flag (without it, a failing assert is silently
swallowed in parallel mode) are in `references/invariants.md` and the runnable
`examples/vault/`.

## Operating procedure (when invoked)

1. Detect the program: read `Anchor.toml` and the IDL (for a native program, read its
   instruction enum and entrypoint instead); list the instructions and the accounts each touches.
2. Scaffold the `trident-tests/fuzz_0/` layout (or use the CLI init flow per
   `references/setup.md`).
3. Define `fuzz_accounts` (signers and PDAs with their real seeds) in
   `fuzz_accounts.rs`; mirror the program's account types in `types.rs`.
4. Write `#[init]` to establish baseline state (create accounts, airdrop, seed PDAs).
5. Write one `#[flow]` per instruction (and per meaningful multi-instruction
   sequence), driving each with random inputs.
6. For every state-changing instruction, write an invariant method using the
   capture-before/after pattern above. Assert the real domain property, not just
   "it didn't error".
7. Run `trident fuzz run fuzz_0 --with-exit-code` from `trident-tests/`. The flag is
   mandatory or invariant failures do not register. Triage a reported failure with the
   printed master seed (`trident fuzz run fuzz_0 <SEED>`), fix, re-run green.

Do not author flow or invariant code from memory. Read the relevant reference file
first; the API is version-specific.

## References (progressive disclosure)

- `references/setup.md`: install, init, `Trident.toml`, prerequisites, run command
- `references/flows.md`: `#[init]`/`#[flow]`/`#[end]`, driving instructions, random inputs, time control
- `references/invariants.md`: capture-before/after, invariant methods, expected-failure handling
- `references/accounts.md`: `fuzz_accounts`, PDA seeds, `get_account_with_type`, discriminator offset
- `references/regression.md`: comparing program behavior across versions

## Worked example

`examples/vault/` is the verified, runnable reference: a tiny Anchor program with a
planted balance-invariant bug plus a Trident suite that catches it (red), and the
`require!` + `checked_sub` fix that makes it pass (green). It is compiled and fuzzed against the exact
v0.12 API; when in doubt about a call, copy from there.

## Provenance

Trident API in this skill is taken from the v0.12.0 source and docs at
`https://github.com/Ackee-Blockchain/trident` (examples under
`examples/hello_world/trident-tests/` and the invariants-assertions documentation).
Re-verify against that source when the pinned version changes.
