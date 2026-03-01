# SideBet

Peer-to-peer social betting on NBA and politics. Bets are settled trustlessly using UMA's Optimistic Oracle V3 on Base.

## Architecture

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   SvelteKit  │───▶│  Rust/Axum   │───▶│  PostgreSQL  │
│   Frontend   │    │   Backend    │    │   Database   │
└──────┬───────┘    └──────┬───────┘    └──────────────┘
       │                   │
       │            ┌──────┴───────┐    ┌──────────────┐
       │            │ Better Auth  │    │    Redis     │
       │            │  (Bun sidecar)│    │   (cache)    │
       │            └──────────────┘    └──────────────┘
       │
       ▼
┌──────────────┐    ┌──────────────┐
│  SideBet.sol │───▶│  UMA Oracle  │
│  (Base L2)   │    │    (OOv3)    │
└──────────────┘    └──────────────┘
```

## Stack

| Layer | Technology |
|---|---|
| Backend | Rust + Axum |
| Database | PostgreSQL (sqlx) |
| Cache | Redis |
| Auth | Better Auth (Bun sidecar) |
| Frontend | SvelteKit + vanilla CSS |
| Contracts | Solidity + Foundry |
| Chain | Base (Coinbase L2) |

## Quick Start

```bash
# 1. Copy env
cp .env.example .env
# Edit .env with your credentials

# 2. Database
createdb sidebet
cd backend && cargo sqlx migrate run

# 3. Auth sidecar
cd auth-sidecar && bun install && bun run src/index.ts

# 4. Backend
cd backend && cargo run

# 5. Frontend
cd frontend && npm install && npm run dev
```

## Project Structure

```
sidebet/
├── backend/          # Rust (Axum) API server
├── auth-sidecar/     # Better Auth (Bun/Hono)
├── contracts/        # Solidity (Foundry) — SideBet.sol
└── frontend/         # SvelteKit app
```

## License

MIT
