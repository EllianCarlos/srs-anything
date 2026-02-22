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

## Local secrets

- `devenv` loads `.env` automatically (`dotenv.enable = true` in `devenv.nix`).
- Copy `.env.example` to `.env` and set a strong local `JWT_SECRET`.
- `.env` / `.env.local` are gitignored and must never be committed.
- CI must define `JWT_SECRET` in repository secrets because backend runs with `SRS_PROFILE=prod`.

## Quality gates

- Lint: `devenv shell -- lint`
- Secrets: `devenv shell -- secrets`
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
4. Backend verifies token and sets `srs_auth` HttpOnly cookie.

## Collaborative LeetCode validation checklist

This step requires your app login session and one API token:

1. Login to LeetCode in your browser.
2. Install `tampermonkey/leetcode-neetcode.user.js`.
3. Open `/integrations` and create an integration API token.
4. Use `Send to Tampermonkey` to bridge token into userscript storage (`GM_setValue`).
5. Do not use app cookie/session auth for ingestion requests.
6. Open/solve a problem page.
7. Confirm event appears on `/integrations` and dashboard latest ingestion card.
