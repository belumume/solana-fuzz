# references/invariants.md: invariants, and the flag that makes them count

The invariant pattern: capture state before, execute, capture state after, assert the expected property in a dedicated method.

```rust
let before = match self.trident.get_account_with_type::<Vault>(&vault, 8) {
    Some(v) => v,                       // 8 = Anchor discriminator size
    None => return,
};
let res = self.trident.process_transaction(&[withdraw_ix], Some("withdraw"));
if res.is_success() {
    if let Some(after) = self.trident.get_account_with_type::<Vault>(&vault, 8) {
        self.withdraw_invariant(&before, &after);
    }
}

fn withdraw_invariant(&self, before: &Vault, after: &Vault) {
    assert!(after.balance <= before.balance, "withdraw must not increase balance");
}
```

- `get_account_with_type::<T>(key, discriminator_size)` reads the account and deserializes `T` after skipping `discriminator_size` bytes. For Anchor accounts that is `8`; for a native program, whatever your layout uses.
- Only check the invariant when `is_success()`. A transaction the program correctly rejected (an over-withdraw on a fixed program) is not a violation; gating on success keeps the green case honest instead of asserting on state that never changed.
- A good invariant takes before/after, asserts one specific property with a descriptive message, and is written as its own method so the intent is named.

## The gotcha that costs an afternoon: `--with-exit-code`

A bare `assert!` panic inside a flow is caught by Trident's flow executor. In the default parallel run, the panicking iteration is **silently dropped** and the process still exits 0, so a violated invariant can look like it passed. The metrics table will read all-success while the invariant is firing every single iteration.

Run with the flag so failures are reported and the exit reflects them:

```bash
trident fuzz run fuzz_0 --with-exit-code
```

With `--with-exit-code`, Trident prints `Fuzzing found failing invariants or unhandled panics` when any invariant fails. Always use it in CI and whenever you want a real pass/fail signal. This is verified behavior: on the worked example, the buggy program reports failing invariants with the flag and the fixed program runs clean, while without the flag both read as "all success".

## Reproduce a specific failure

The run prints a `MASTER SEED`. Replay it deterministically:

```bash
trident fuzz run fuzz_0 <SEED>          # rerun the exact sequence
trident fuzz debug fuzz_0 <SEED>        # single-threaded replay with detail
```

Single-threaded / debug replays also surface the per-assertion location and message that parallel mode suppresses.
