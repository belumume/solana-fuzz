# references/flows.md: flows, driving instructions, random inputs

A fuzz test is a struct deriving `FuzzTestMethods` (a `Trident` client plus the generated `AccountAddresses`) and a `#[flow_executor]` impl. Trident generates the instruction builders from your IDL into `types.rs`; you write the flows.

## Lifecycle per iteration

```rust
#[flow_executor]
impl FuzzTest {
    #[init] fn start(&mut self) { /* runs once at the start of each iteration */ }
    #[flow] fn flow_a(&mut self) { /* one is selected at random each iteration */ }
    #[flow] fn flow_b(&mut self) { /* add as many as you need */ }
    #[end]  fn end(&mut self)   { /* runs once at the end of each iteration */ }
}
```

Trident resets program state between iterations, so each iteration is independent.

## Driving an instruction

Build from the generated `types::<program>::*` builders, then submit:

```rust
let ix = WithdrawInstruction::data(WithdrawInstructionData::new(amount))
    .accounts(WithdrawInstructionAccounts::new(vault, authority))
    .instruction();
let res = self.trident.process_transaction(&[ix], Some("withdraw"));
```

`process_transaction(&[ix], Some("label"))` returns a result with `.is_success()`. Pass several instructions in one `&[a, b, c]` call to fuzz a multi-instruction sequence atomically.

## Random inputs

```rust
let amount = self.trident.random_from_range(0..2_000_000u64);  // uniform over the range
self.trident.airdrop(&authority, 10 * LAMPORTS_PER_SOL);       // fund a signer
```

Choose ranges that straddle the program's boundaries. The vault bug only triggers when `amount > balance`, so the worked example deposits `0..1_000_000` but withdraws `0..2_000_000`; that overlap makes the fuzzer hit both the valid path and the overflow path. A withdraw range of `0..u64::MAX` still finds the bug, but then every withdraw on the *fixed* program is rejected, so the green run never exercises a real withdrawal. Overlapping ranges keep both red and green meaningful.

## Self-contained flow vs cross-flow state

The simplest and most reliable pattern is one self-contained scenario per flow: allocate fresh accounts, run the full setup plus the operation under test, then check the invariant. That avoids any ordering assumptions between flows. See `../examples/vault/trident-tests/fuzz_0/test_fuzz.rs`. For genuinely stateful sequences, allocate the accounts in `#[init]` and reuse them across flows within the iteration.
