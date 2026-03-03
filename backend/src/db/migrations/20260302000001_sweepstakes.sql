-- Sweepstakes migration: replace crypto/blockchain with virtual coins

-- ── Users ──
ALTER TABLE users DROP COLUMN IF EXISTS wallet_address;
ALTER TABLE users ADD COLUMN IF NOT EXISTS coin_balance BIGINT NOT NULL DEFAULT 1000;
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_daily_bonus TIMESTAMPTZ;

-- Rename total_wagered_wei → total_wagered (was TEXT from prior migration, convert to BIGINT)
ALTER TABLE users ALTER COLUMN total_wagered_wei DROP DEFAULT;
ALTER TABLE users ALTER COLUMN total_wagered_wei TYPE BIGINT USING COALESCE(total_wagered_wei::BIGINT, 0);
ALTER TABLE users ALTER COLUMN total_wagered_wei SET DEFAULT 0;
ALTER TABLE users RENAME COLUMN total_wagered_wei TO total_wagered;

DROP INDEX IF EXISTS idx_users_wallet;

-- ── Bets ──
-- Rename amount_wei → amount (was TEXT from prior migration, convert to BIGINT)
ALTER TABLE bets ALTER COLUMN amount_wei DROP DEFAULT;
ALTER TABLE bets ALTER COLUMN amount_wei TYPE BIGINT USING COALESCE(amount_wei::BIGINT, 0);
ALTER TABLE bets ALTER COLUMN amount_wei SET DEFAULT 0;
ALTER TABLE bets RENAME COLUMN amount_wei TO amount;

ALTER TABLE bets DROP COLUMN IF EXISTS on_chain_bet_id;
ALTER TABLE bets DROP COLUMN IF EXISTS contract_address;
ALTER TABLE bets DROP COLUMN IF EXISTS assertion_id;

ALTER TABLE bets ADD COLUMN IF NOT EXISTS winner_id UUID REFERENCES users(id);

-- ── Transactions (coin ledger) ──
CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    type VARCHAR(24) NOT NULL,
    amount BIGINT NOT NULL,
    balance_after BIGINT NOT NULL,
    reference_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_transactions_user ON transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions(type);
