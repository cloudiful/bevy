# AGENTS.md

Repository rules for `/Volumes/Enterprise/codes/cloudiful-bevy`.

## CI policy

- This repo uses explicit crate matrices in `.gitea/workflows/ci.yml`.
- Every time a new workspace crate is added, update both CI matrices in `.gitea/workflows/ci.yml` in same change:
  - `jobs.test.strategy.matrix.include`
  - `jobs.publish.strategy.matrix.include`
- If a crate has runnable examples that should be checked in CI, declare them in the test matrix with an `example` field.
- If a crate has no example to check, set `example: ""` in its matrix entry and keep the conditional example step intact.

## Workspace policy

- Keep `[workspace].members` in `Cargo.toml` aligned with CI matrix entries.
- New reusable crates should document scope and non-goals in their own `README.md`.

## Guidance for Codex and other agents

- Yes, put repo-local workflow rules here. Codex reads `AGENTS.md` automatically when working in this repo.
- Keep stable, agent-facing instructions in `AGENTS.md`.
- Keep user-facing crate usage docs in each crate `README.md`.
