# SideBet

SideBet is a social sweepstakes betting app for sports and events. It runs as three services sharing one Postgres database:

- `frontend/` - SvelteKit UI
- `backend/` - Rust + Axum API
- `auth-sidecar/` - Better Auth on Bun

All betting uses virtual coins only. No real money and no crypto settlement.

## Repo shortcuts

The root package now exposes a few convenience commands so you do not have to hop between folders for common checks:

```bash
npm run dev:frontend
npm run build:frontend
npm run check:frontend
npm run check:auth
npm run test:backend
```

## Production shape

Recommended topology:

- `frontend` served on the public app origin
- `backend` on the API origin
- `auth-sidecar` on the auth origin
- all services share the same Postgres database

For easiest cookie handling in production, keep the services behind the same parent domain and set `AUTH_COOKIE_DOMAIN` when you need cross-subdomain cookies.

## Deployment plan

This repo is deployable, but not as a single Vercel app. The production shape is:

- `frontend/` on Vercel
- `backend/` on a container host
- `auth-sidecar/` on a container host
- one managed Postgres instance shared by `backend` and `auth-sidecar`

The frontend can now build for either target:

- default `npm run build` inside `frontend/` still uses `@sveltejs/adapter-node` for Docker and local Node hosting
- Vercel builds use `@sveltejs/adapter-vercel`

### Frontend on Vercel

Create one Vercel project for `frontend/` only.

1. Import the repository into Vercel
2. Set the project's Root Directory to `frontend`
3. Add these environment variables in Vercel for both Preview and Production as needed:
   - `PUBLIC_API_URL`
   - `PUBLIC_AUTH_URL`
   - `PUBLIC_WS_URL`
   - optional `DEPLOY_TARGET=vercel` if you disable Vercel system environment variables
4. Deploy

Use a custom domain such as `app.example.com` for the frontend if you want cookie-based auth to work cleanly with `api.example.com` and `auth.example.com`.

### Backend and auth on a container platform

`backend/` and `auth-sidecar/` are long-running services, so deploy them to a platform that can run the existing Dockerfiles.

- backend image: `Dockerfile.backend`
- auth image: `Dockerfile.auth`

Important runtime settings:

- both services need the same `DATABASE_URL`
- `backend` needs `FRONTEND_URL` set to the frontend origin
- `auth-sidecar` needs `BETTER_AUTH_URL`, `BACKEND_URL`, `FRONTEND_URL`, `AUTH_SECRET`, and the same `DATABASE_URL`
- `auth-sidecar` now accepts the platform-provided `PORT` env var automatically, while still honoring `AUTH_PORT` when you set it explicitly

### Preview deploy caveat

Preview deployments on Vercel get their own preview URLs. If you want authenticated preview environments to work against shared backend/auth services, make sure those services allow the preview frontend origin in `FRONTEND_URL`, or use a custom-domain setup that keeps all services under the same parent domain.

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

The repo root `.env` is used by Docker Compose and mirrors the service-local defaults. The individual service folders also include local `.env` files for direct `cargo`, `bun`, and `vite` startup.

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
