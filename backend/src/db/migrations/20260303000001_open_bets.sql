-- Allow bets without a pre-selected opponent (open/marketplace bets)
ALTER TABLE bets ALTER COLUMN opponent_id DROP NOT NULL;
