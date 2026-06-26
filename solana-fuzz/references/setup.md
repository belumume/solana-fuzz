# references/setup.md: install, init, run, and the gotchas

Trident is a Linux/macOS tool. On Windows, build inside WSL2 (Ubuntu), not native Windows. The Solana SBF toolchain (`cargo-build-sbf`), Anchor, and Trident do not run natively on Windows.

## Toolchain (verified working set, 2026-06)

| Tool | Version used | Install |
|---|---|---|
| Rust | 1.86.0 (host) | `rustup` (https://rustup.rs) |
| solana-cli | 2.1.19 (Agave) | `sh -c "$(curl -sSfL https://release.anza.xyz/v2.1.19/install)"` |
| Anchor | 0.31.0/0.31.1 | `avm install 0.31.1 && avm use 0.31.1` |
| Trident | 0.12.0 | `cargo install trident-cli --locked` |

`anchor build` invokes `cargo-build-sbf`, which uses the Solana platform-tools' **own bundled Rust (1.79 for solana 2.1.x)**, NOT your host rustup toolchain. This split is the source of the most common build failure (below).

## Workflow

```bash
anchor init <name>                 # scaffold the Anchor workspace
# ... write the program, then:
anchor build                       # compiles to SBF + generates target/idl/<name>.json
trident init                       # adds Trident.toml + trident-tests/fuzz_0/ (reads the IDL)
trident fuzz run fuzz_0 --with-exit-code   # run the fuzzer (--with-exit-code is mandatory; see invariants.md)
trident fuzz debug fuzz_0 <SEED>   # replay a single crashing seed
trident fuzz add                   # add another fuzz test template
```

`trident init` builds the program first; pass `--skip-build` if you already ran `anchor build`. Use `-p <program>` to target a specific program in a multi-program workspace.

## Gotcha 1 (very common): `feature 'edition2024' is required`

```
error: failed to parse manifest at `.../crypto-common-0.2.2/Cargo.toml`
Caused by: feature `edition2024` is required ... not stabilized in this version of Cargo (1.79.0)
```

Cause: cargo resolves a transitive dependency to its newest version, which requires `edition2024` (Cargo 1.85+), but the SBF platform-tools Rust is 1.79. The build dies during dependency resolution, before any of your code compiles.

Fix: pin the offending crate DOWN to a version that predates edition2024. Find the chain, then pin the top of it. The 2026-06 offender is `crypto-common 0.2.2 <- digest 0.11 <- blake3 1.8.x <- solana-program`:

```bash
cargo tree -i crypto-common@0.2.2      # find what pulls it
cargo update -p blake3 --precise 1.5.5 # blake3 1.5.x uses digest 0.10 / crypto-common 0.1.7
```

This is version-volatile: the specific crate and pin change as the ecosystem moves. The procedure is stable: read the `edition2024` error, `cargo tree -i <crate>` to find the chain, `cargo update -p <top-of-chain> --precise <older>` until the edition2024 crate is gone. Re-run `anchor build`; expect to repeat for more than one crate in a bad month. The single most effective pin is `cargo update -p solana-program --precise <older 2.1.x>`, which reverts most of the solana-* transitive graph at once.

Two ways to handle it, pick by goal:

- **Pin down (best for a shippable, reproducible example):** commit the resulting `Cargo.lock`. The example then builds identically for any reviewer on the same platform-tools, no surprises. This is what this repo's worked example does.
- **Use a newer platform-tools (best to avoid pins entirely):** the older platform-tools Rust is the real cause. `cargo-build-sbf` and `anchor build` accept `--tools-version vX.Y` to fetch a newer platform-tools whose Rust supports edition2024. As of 2026-06 the installed default is v1.43 (Rust 1.79); the latest is v1.54. Example: `cargo-build-sbf --tools-version v1.54`, or set it in `Anchor.toml`/the build invocation. A platform-tools with Rust >= 1.85 parses edition2024 natively, so no pins are needed and you can build against current solana-program.

## Gotcha 2: build where the filesystem is fast

Building under `/mnt/c/...` (a Windows path mounted in WSL) is very slow because of cross-filesystem I/O. Build in the WSL-native filesystem (`~/...`) and copy the source into your repo, or keep the whole repo in WSL.

## Gotcha 3: keypairs and target/ never get committed

`anchor init` writes a program keypair under `target/deploy/<name>-keypair.json`. Gitignore `target/`, `.anchor/`, `test-ledger/`, and `*-keypair.json`. Committing a program keypair leaks the upgrade authority of a deployed program.
