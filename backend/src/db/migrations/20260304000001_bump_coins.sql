-- Increase starting coins: 1000 → 10000
ALTER TABLE users ALTER COLUMN coin_balance SET DEFAULT 10000;

-- Give existing users with the old default (1000) who haven't placed any bets the new amount
UPDATE users SET coin_balance = 10000
  WHERE coin_balance = 1000 AND total_wagered = 0;
