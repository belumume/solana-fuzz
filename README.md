# solana-fuzz

An MIT-licensed Claude Code / Codex skill that writes [Trident](https://github.com/Ackee-Blockchain/trident) property-based fuzz tests and invariant checks for Solana programs (Anchor and native).

Most Solana programs ship with happy-path example tests and nothing else. Fuzzing, the part that surfaces the runtime logic bugs, is the piece that usually gets left manual, so most teams skip it. Trident is the Solana Foundation supported fuzzer from Ackee Blockchain, but wiring it up by hand for every program is real work. This skill does that wiring: point an agent at a program and its IDL, and it scaffolds the Trident harness, writes the per-instruction flows that drive each instruction with random input, and writes the capture-before/after invariant checks that assert the program's state stays correct.

This is dynamic, dev-loop property testing. It is not a static auditor and it does not replace example-based tests (LiteSVM, Mollusk, Surfpool); it complements them.

## The skill

The skill lives in [`solana-fuzz/SKILL.md`](solana-fuzz/SKILL.md) with progressive-disclosure reference files under [`solana-fuzz/references/`](solana-fuzz/references/):

- `setup.md`: toolchain, init, run, and the real build gotchas (edition2024 / MSRV / platform-tools)
- `flows.md`: `#[init]` / `#[flow]` / `#[end]`, driving instructions, random inputs
- `invariants.md`: the capture-before/after pattern, expected-failure handling
- `accounts.md`: fuzz accounts, PDA seeds, reading typed state
- `regression.md`: comparing behavior across program versions

Install it with the bundled [`install.sh`](install.sh) (copies `solana-fuzz/` into your skills directory; set `SKILLS_DIR=...` to choose where, `WITH_TRIDENT=1` to also `cargo install trident-cli --locked`). Or drop `solana-fuzz/` into your skills directory by hand, or reference `SKILL.md` directly.

## Worked example: a vault the fuzzer breaks

[`solana-fuzz/examples/vault/`](solana-fuzz/examples/vault/) is a complete, runnable proof. It is a tiny Anchor program with a bug planted on purpose, plus a Trident suite that catches it.

The bug is in `withdraw`: it subtracts without checking the balance.

```rust
// programs/vault/src/lib.rs  (buggy)
pub fn withdraw(ctx: Context<Update>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    vault.balance = vault.balance.wrapping_sub(amount); // underflows when amount > balance
    Ok(())
}
```

The invariant is one line: a withdraw must never increase the balance.

```rust
// trident-tests/fuzz_0/test_fuzz.rs  (invariant)
assert!(after.balance <= before.balance, "withdraw must not increase balance");
```

The fuzzer drives `deposit` and `withdraw` with random amounts. As soon as it draws a withdraw amount larger than the balance, the `wrapping_sub` wraps to a huge number, the balance jumps up, and the invariant fails with the exact crashing seed.

Real output from this example. The buggy program (red), the fuzzer finds it:

```text
| Instruction | Invoked Total | Ix Success | Ix Failed | Instruction Panicked |
| deposit     | 1299          | 1299       | 0         | 0                    |
| withdraw    | 1299          | 1299       | 0         | 0                    |
Error: Fuzzing found failing invariants or unhandled panics
```

The same fuzz run after the one-line fix (green): 24,706 valid withdrawals pass the invariant, every over-withdrawal is correctly rejected, zero violations:

```text
| Instruction | Invoked Total | Ix Success | Ix Failed | Instruction Panicked |
| withdraw    | 99200         | 24706      | 74494     | 0                    |
```

The one-command proof, reproducible on a fresh clone:

```bash
cd solana-fuzz/examples/vault
anchor build                      # build the program (see solana-fuzz/references/setup.md for toolchain)
cd trident-tests
trident fuzz run fuzz_0 --with-exit-code   # finds the planted invariant violation (red)
```

Fixing `withdraw` to `require!(amount <= vault.balance)` + `checked_sub` makes the same fuzz run pass clean (green). That green-to-red flip on one command is the whole point: the skill produces tests that catch real logic bugs example tests never reach.

## What is verified, and what is a documented pattern

The vault example above is verified end to end: it compiles and fuzzes against the exact Trident v0.12.0 API, red on the planted bug and green on the fix. The skill's other capabilities -- native (non-Anchor) programs, multi-instruction sequences, and the across-versions regression flow ([`regression.md`](solana-fuzz/references/regression.md)) -- are documented patterns, not separately shipped runnable examples. Apply them to your program and your installed Trident version, and verify the generated tests the same way: run them, confirm a real bug goes red and the fix goes green. The skill's own instructions reinforce this -- they tell the agent to read the version-specific reference and verify against the Trident source rather than author from memory.

## Toolchain

Trident is a Linux/macOS tool; on Windows build inside WSL2. The verified working set: Rust (host) + solana-cli 2.1.19 + Anchor 0.31 + Trident 0.12. The Solana SBF build has a known `edition2024` / MSRV friction with the 2026 crates.io state; the fixes (dependency pins or a newer platform-tools via `--tools-version`) are documented in [`solana-fuzz/references/setup.md`](solana-fuzz/references/setup.md). The example ships a committed `Cargo.lock` so it builds reproducibly.

## License

MIT. See [LICENSE](LICENSE).
