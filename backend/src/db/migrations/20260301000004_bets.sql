-- Bets table
CREATE TABLE IF NOT EXISTS bets (
    id UUID PRIMARY KEY,
    creator_id UUID NOT NULL REFERENCES users(id),
    opponent_id UUID NOT NULL REFERENCES users(id),
    event_id UUID REFERENCES events(id),
    question TEXT NOT NULL,
    creator_position TEXT NOT NULL,
    opponent_position TEXT NOT NULL,
    amount_wei BIGINT NOT NULL,
    odds_numerator INT NOT NULL,
    odds_denominator INT NOT NULL,
    reference_odds JSONB,
    status VARCHAR(16) NOT NULL DEFAULT 'proposed',
    on_chain_bet_id BIGINT,
    contract_address VARCHAR(42),
    assertion_id VARCHAR(66),
    outcome VARCHAR(16),
    resolved_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_bets_creator ON bets(creator_id);
CREATE INDEX IF NOT EXISTS idx_bets_opponent ON bets(opponent_id);
CREATE INDEX IF NOT EXISTS idx_bets_status ON bets(status);
CREATE INDEX IF NOT EXISTS idx_bets_event ON bets(event_id) WHERE event_id IS NOT NULL;
