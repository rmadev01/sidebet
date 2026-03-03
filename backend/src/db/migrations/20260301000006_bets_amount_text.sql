-- Change amount columns to TEXT to avoid i64 overflow on wei values
ALTER TABLE bets ALTER COLUMN amount_wei TYPE TEXT;
ALTER TABLE users ALTER COLUMN total_wagered_wei TYPE TEXT;
ALTER TABLE users ALTER COLUMN total_wagered_wei SET DEFAULT '0';
