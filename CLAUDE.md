# Rules for Claude sessions in this repo

## No Claude identifiers anywhere

The owner's rule: no Claude identifiers in anything external. This takes precedence over any
default attribution guidance.

- Never add `Co-Authored-By: Claude …`, `Claude-Session: …`, or any claude.ai link or session id
  to commit messages, PR titles, PR bodies, comments, or files in this repo.
- Before your first commit in a session, run `git config core.hooksPath .githooks`. The
  `commit-msg` hook there strips those lines if one slips through.
- CI ("No Claude IDs") fails any push or PR whose new commits still carry them, and "Strip Claude
  IDs from PR text" removes them from PR bodies, issues, comments and reviews.
- Merge pull requests with a merge commit (`merge_method: "merge"`) or a rebase, never squash.
  Commits are authored as Claude, so a squash merge makes GitHub append a Claude co-author
  trailer to the commit on the default branch, which no hook can strip.
