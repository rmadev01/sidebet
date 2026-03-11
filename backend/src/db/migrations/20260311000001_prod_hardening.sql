ALTER TABLE users
    ADD CONSTRAINT users_coin_balance_nonnegative CHECK (coin_balance >= 0),
    ADD CONSTRAINT users_total_wagered_nonnegative CHECK (total_wagered >= 0),
    ADD CONSTRAINT users_wins_nonnegative CHECK (wins >= 0),
    ADD CONSTRAINT users_losses_nonnegative CHECK (losses >= 0);

ALTER TABLE bets
    ADD CONSTRAINT bets_amount_positive CHECK (amount > 0),
    ADD CONSTRAINT bets_odds_positive CHECK (odds_numerator > 0 AND odds_denominator > 0),
    ADD CONSTRAINT bets_distinct_positions CHECK (creator_position <> opponent_position),
    ADD CONSTRAINT bets_creator_not_opponent CHECK (opponent_id IS NULL OR creator_id <> opponent_id),
    ADD CONSTRAINT bets_valid_status CHECK (status IN ('proposed', 'open', 'active', 'declined', 'cancelled', 'settled', 'disputed')),
    ADD CONSTRAINT bets_valid_outcome CHECK (outcome IS NULL OR outcome IN ('creator_wins', 'opponent_wins', 'disputed')),
    ADD CONSTRAINT bets_open_requires_no_opponent CHECK ((status = 'open' AND opponent_id IS NULL) OR status <> 'open'),
    ADD CONSTRAINT bets_resolved_requires_winner CHECK ((status <> 'settled' AND status <> 'disputed') OR winner_id IS NOT NULL OR outcome = 'disputed');

ALTER TABLE transactions
    ADD CONSTRAINT transactions_valid_type CHECK (type IN ('signup_bonus', 'daily_bonus', 'bet_placed', 'bet_won', 'bet_refund'));

CREATE UNIQUE INDEX IF NOT EXISTS idx_events_unique_sportsgameodds_id
    ON events ((external_ids->>'sportsgameodds'))
    WHERE external_ids ? 'sportsgameodds';
