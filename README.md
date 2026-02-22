# SRS Anything MVP

Rust backend + React frontend to track LeetCode/NeetCode activity and schedule spaced repetition.

## Stack

- Backend: Rust, Axum, Tokio
- Frontend: React, Mantine, React Query, React Router
- Dev env: `devenv` + `direnv`
- Coverage: `cargo-llvm-cov` + Vitest coverage

## Quick start (NixOS)

1. Install and allow direnv for this repo:
   - `direnv allow`
2. Start the backend:
   - `devenv shell -- sh -c "cd backend && cargo run"`
3. Start the frontend:
   - `devenv shell -- sh -c "cd frontend && npm run dev"`

## Quality gates

- Lint: `devenv shell -- lint`
- Tests: `devenv shell -- test`
- Coverage: `devenv shell -- coverage`

## SRS schedule configuration

The backend reads SRS intervals from `backend/config/srs_schedule.yaml`:

- `active_profile`: default profile when `SRS_PROFILE` is not set.
- `profiles.<name>.unit`: `days` or `minutes`.
- `profiles.<name>.intervals`: list of positive integers used for spacing.

Environment overrides:

- `SRS_CONFIG_PATH`: optional path to a custom schedule YAML file.
- `SRS_PROFILE`: optional profile name to override `active_profile`.

Fallback rules:

- Missing/invalid YAML file falls back to built-in production schedule.
- Unknown profile falls back to built-in production schedule.

Local `devenv` defaults to `SRS_PROFILE=test` for faster review loops, while CI forces `SRS_PROFILE=prod`.

## MVP auth flow

1. Open frontend at `http://localhost:5173/login`.
2. Enter email and request magic link.
3. Use generated dev token to open verify page.
4. Session token is stored in browser localStorage.

## Collaborative LeetCode validation checklist

This step requires your real account session:

1. Login to LeetCode in your browser.
2. Install `tampermonkey/leetcode-neetcode.user.js`.
3. Copy the real `srs_session_token` from the SRS app tab (after `/verify`) into LeetCode/NeetCode localStorage.
4. Do not use any placeholder or hint token shown on integrations pages as auth.
5. Open/solve a problem page.
6. Confirm event appears on `/integrations` and dashboard latest ingestion card.
