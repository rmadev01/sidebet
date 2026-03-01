-- Events table
CREATE TABLE IF NOT EXISTS events (
    id UUID PRIMARY KEY,
    category VARCHAR(16) NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    sport VARCHAR(32),
    league VARCHAR(32),
    starts_at TIMESTAMPTZ,
    external_ids JSONB NOT NULL DEFAULT '{}',
    cached_odds JSONB NOT NULL DEFAULT '{}',
    odds_updated_at TIMESTAMPTZ,
    status VARCHAR(16) NOT NULL DEFAULT 'upcoming',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_events_category ON events(category);
CREATE INDEX IF NOT EXISTS idx_events_status ON events(status);
CREATE INDEX IF NOT EXISTS idx_events_starts_at ON events(starts_at);
