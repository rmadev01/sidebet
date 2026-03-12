# SideBet

SideBet is a social sweepstakes betting app for sports and events. It runs as three services sharing one Postgres database:

- `frontend/` - SvelteKit UI
- `backend/` - Rust + Axum API
- `auth-sidecar/` - Better Auth on Bun

All betting uses virtual coins only. No real money and no crypto settlement.

## Production shape

Recommended topology:

- `frontend` served on the public app origin
- `backend` on the API origin
- `auth-sidecar` on the auth origin
- all services share the same Postgres database

For easiest cookie handling in production, keep the services behind the same parent domain and set `AUTH_COOKIE_DOMAIN` when you need cross-subdomain cookies.

## Required secrets and keys

Copy `.env.example` to `.env` and replace the placeholders for:

- `DATABASE_URL`
- `AUTH_SECRET`
- `SPORTSGAMEODDS_API_KEY`
- optional social login keys if you enable Google or Discord auth

## Local development

1. Create env file

```bash
cp .env.example .env
```

2. Start Postgres

```bash
createdb sidebet
```

3. Start auth sidecar

```bash
cd auth-sidecar
bun install
bun run src/index.ts
```

The auth sidecar auto-creates the Better Auth tables it needs on startup.

4. Start backend

```bash
cd backend
cargo run
```

The backend auto-runs SQL migrations at startup.

5. Start frontend

```bash
cd frontend
npm install
npm run dev
```

## Verification

Run the checks before shipping:

```bash
cd backend && cargo test
cd frontend && npm run check
cd frontend && npm run test:smoke
cd auth-sidecar && npm install && npm run check
```

There is also an integration helper script at `scripts/seed_and_test.sh` for end-to-end local flow testing once all services are running.

## Containerized startup

Build and run the full stack with Docker Compose:

```bash
docker compose up --build
```

The frontend image reads `PUBLIC_API_URL`, `PUBLIC_AUTH_URL`, and `PUBLIC_WS_URL` at build time, so make sure they are set in the root `.env` before building.

Compose overrides the backend and auth `DATABASE_URL` values to use the internal `postgres` hostname, so you can keep the root `.env` on localhost-oriented values for browser access while still using the same file for containers.

Services exposed locally:

- frontend: `http://localhost:5173`
- backend: `http://localhost:3000`
- auth: `http://localhost:3001`
- postgres: `localhost:5432`

## Current operational rules

- auth requests are rate-limited in-process
- backend mutating routes enforce origin checks for cookie-backed auth
- active bets can only be resolved from event results or marked disputed for refund
- event sync uses advisory locks and conflict-safe upserts

## Notes for image-based deploys

If your goal is "set a couple keys and spin up an image", the critical runtime inputs are:

- one reachable Postgres instance
- `AUTH_SECRET`
- `SPORTSGAMEODDS_API_KEY`
- correct public service URLs
- optional `AUTH_COOKIE_DOMAIN` for shared cookies across subdomains

Once those are present, the auth sidecar bootstraps auth tables and the backend bootstraps app migrations automatically.
