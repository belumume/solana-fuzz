# references/regression.md: regression fuzzing across versions

Trident can record per-seed results and diff two runs, so you catch behavior that changed between two builds of the same program (a refactor that quietly altered an outcome, or a fix that changed more than intended).

```bash
# 1. record a baseline (regression output enabled via Trident.toml [fuzz] or the env var)
FUZZING_REGRESSION=true trident fuzz run fuzz_0 <SEED>
# 2. change the program, rebuild
anchor build
# 3. record again with the SAME master seed
FUZZING_REGRESSION=true trident fuzz run fuzz_0 <SEED>
# 4. diff the two regression JSON files
trident compare <baseline.json> <new.json>
```

`trident compare` reports the iteration seeds whose outcome differs between the two files. Keep the master seed fixed across both runs (pass it as the optional `[SEED]` argument) so the comparison is apples-to-apples; otherwise the two runs explore different inputs and every seed "differs".

Use it to confirm a fix changed only the intended behavior, or to gate a refactor in CI: a non-empty diff on an unrelated change is a regression to investigate.

This surface (the `[fuzz]` regression config, the env var name, the JSON format) is version-specific. Re-read it against the installed Trident with `trident fuzz --help` and `trident compare --help` before relying on the exact flags.
