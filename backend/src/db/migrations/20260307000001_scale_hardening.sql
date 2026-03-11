CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Remove reciprocal duplicate friendship rows before enforcing canonical uniqueness.
WITH ranked AS (
    SELECT id,
           ROW_NUMBER() OVER (
               PARTITION BY LEAST(requester_id, addressee_id), GREATEST(requester_id, addressee_id)
               ORDER BY created_at ASC, id ASC
           ) AS row_num
    FROM friendships
)
DELETE FROM friendships
WHERE id IN (
    SELECT id FROM ranked WHERE row_num > 1
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_friendships_unique_pair
    ON friendships (LEAST(requester_id, addressee_id), GREATEST(requester_id, addressee_id));

CREATE INDEX IF NOT EXISTS idx_bets_creator_created_at
    ON bets (creator_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_bets_opponent_created_at
    ON bets (opponent_id, created_at DESC)
    WHERE opponent_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_bets_status_updated_at
    ON bets (status, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_bets_open_feed
    ON bets (expires_at, created_at DESC)
    WHERE status = 'open' AND opponent_id IS NULL;

CREATE INDEX IF NOT EXISTS idx_events_sportsgameodds_id
    ON events ((external_ids->>'sportsgameodds'));

CREATE INDEX IF NOT EXISTS idx_transactions_user_created_at
    ON transactions (user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_users_username_trgm
    ON users USING gin (username gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_users_display_name_trgm
    ON users USING gin (display_name gin_trgm_ops);
