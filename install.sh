#!/usr/bin/env bash
# solana-fuzz installer: copy the skill into your agent's skills directory,
# and optionally install the Trident CLI it generates tests for.
set -euo pipefail

SKILL_NAME="solana-fuzz"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRC_DIR="${HERE}/${SKILL_NAME}"

# Target skills directory. Default is the Claude Code location; override with SKILLS_DIR=...
SKILLS_DIR="${SKILLS_DIR:-${HOME}/.claude/skills}"

if [ ! -d "${SRC_DIR}" ]; then
  echo "error: ${SKILL_NAME}/ not found next to this script (${SRC_DIR})" >&2
  exit 1
fi

mkdir -p "${SKILLS_DIR}"
cp -R "${SRC_DIR}" "${SKILLS_DIR}/"
echo "installed: ${SKILLS_DIR}/${SKILL_NAME}"

# Optional: install the Trident CLI, pinned via --locked (set WITH_TRIDENT=1).
if [ "${WITH_TRIDENT:-0}" = "1" ]; then
  if command -v cargo >/dev/null 2>&1; then
    echo "installing trident-cli --locked (this can take a while)..."
    cargo install trident-cli --locked
  else
    echo "skipped trident-cli: cargo not on PATH; install Rust first" >&2
  fi
fi

echo "done. Trident itself runs on Linux/WSL2 with solana-cli + Anchor; see ${SKILL_NAME}/references/setup.md"
