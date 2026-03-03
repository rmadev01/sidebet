<script lang="ts">
  import { onMount } from 'svelte';
  import { user, isAuthenticated, coinBalance } from '$lib/stores';
  import { getMe, getBalance, claimDailyBonus, getTransactions } from '$lib/api';

  let profile = $state<any>(null);
  let wallet = $state<any>(null);
  let transactions = $state<any[]>([]);
  let loading = $state(true);
  let claiming = $state(false);

  onMount(async () => {
    if (!$isAuthenticated) { loading = false; return; }
    try {
      const [me, w, txns] = await Promise.all([
        getMe(),
        getBalance(),
        getTransactions(20),
      ]);
      profile = me;
      wallet = w;
      transactions = txns;
    } catch { /* ignore */ }
    loading = false;
  });

  function formatCoins(n: number) { return n?.toLocaleString() ?? '0'; }

  function winRate() {
    if (!profile) return 0;
    const total = profile.wins + profile.losses;
    return total > 0 ? Math.round((profile.wins / total) * 100) : 0;
  }

  async function doClaim() {
    claiming = true;
    try {
      const res = await claimDailyBonus();
      coinBalance.set(res.new_balance);
      wallet.coin_balance = res.new_balance;
      wallet.bonus_available = false;
    } catch { /* already claimed */ }
    claiming = false;
  }

  function typeLabel(t: string) {
    const map: Record<string,string> = {
      signup_bonus: '🎁 Signup Bonus',
      daily_bonus: '🎁 Daily Bonus',
      bet_placed: '🎲 Bet Placed',
      bet_won: '🏆 Bet Won',
      bet_refund: '↩️ Refund',
    };
    return map[t] || t;
  }
</script>

<svelte:head>
  <title>Profile — SideBet</title>
</svelte:head>

<div class="profile-page">
  {#if loading}
    <p class="empty-state">Loading…</p>
  {:else if !profile}
    <p class="empty-state">Sign in to view your profile.</p>
  {:else}
    <!-- Header -->
    <div class="profile-header animate-in">
      <div class="profile-av">
        <span class="av av-2 av--lg">{profile.display_name?.[0]?.toUpperCase() || '?'}</span>
      </div>
      <div class="profile-info">
        <h1>{profile.display_name}</h1>
        <span class="handle">@{profile.username}</span>
      </div>
    </div>

    <!-- Wallet Card -->
    <div class="wallet-card animate-in" style="animation-delay: 80ms">
      <div class="wallet-balance">
        <span class="wallet-label">Coin Balance</span>
        <span class="wallet-amount mono">{formatCoins(wallet?.coin_balance || 0)}</span>
      </div>
      {#if wallet?.bonus_available}
        <button class="btn btn-primary" onclick={doClaim} disabled={claiming}>
          {claiming ? 'Claiming…' : 'Claim Daily Bonus (+100)'}
        </button>
      {:else}
        <span class="wallet-sub">Daily bonus claimed ✓</span>
      {/if}
    </div>

    <!-- Stats -->
    <div class="stats-grid animate-in" style="animation-delay: 160ms">
      <div class="stat-card">
        <span class="stat-val lime">{winRate()}%</span>
        <span class="stat-label">Win Rate</span>
      </div>
      <div class="stat-card">
        <span class="stat-val">{profile.wins}</span>
        <span class="stat-label">Wins</span>
      </div>
      <div class="stat-card">
        <span class="stat-val">{profile.losses}</span>
        <span class="stat-label">Losses</span>
      </div>
      <div class="stat-card">
        <span class="stat-val mono">{formatCoins(profile.total_wagered)}</span>
        <span class="stat-label">Total Wagered</span>
      </div>
    </div>

    <!-- Transaction History -->
    {#if transactions.length > 0}
      <section class="section animate-in" style="animation-delay: 240ms">
        <h2>Recent Transactions</h2>
        <div class="txn-list">
          {#each transactions as txn}
            <div class="txn-row">
              <span class="txn-type">{typeLabel(txn.type)}</span>
              <span class="txn-amount mono" class:positive={txn.amount > 0} class:negative={txn.amount < 0}>
                {txn.amount > 0 ? '+' : ''}{formatCoins(txn.amount)}
              </span>
            </div>
          {/each}
        </div>
      </section>
    {/if}
  {/if}
</div>

<style>
  .profile-page { max-width: 640px; }

  .empty-state {
    color: var(--text-3);
    font-size: 0.875rem;
    padding: 40px 0;
    text-align: center;
  }

  .profile-header {
    display: flex;
    align-items: center;
    gap: 20px;
    margin-bottom: 24px;
  }
  .profile-info h1 { font-size: 1.5rem; margin-bottom: 2px; }
  .handle { font-size: 0.875rem; color: var(--text-3); }

  .av--lg {
    width: 64px;
    height: 64px;
    font-size: 1.5rem;
  }

  /* Wallet Card */
  .wallet-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: linear-gradient(135deg, hsl(75 70% 12%), hsl(75 60% 8%));
    border: 1px solid var(--lime-dim);
    border-radius: var(--r-lg);
    padding: 24px;
    margin-bottom: 24px;
  }
  .wallet-label {
    display: block;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-3);
    margin-bottom: 4px;
  }
  .wallet-amount {
    font-size: 1.75rem;
    font-weight: 800;
    color: var(--lime);
  }
  .wallet-sub {
    font-size: 0.8125rem;
    color: var(--text-3);
  }

  /* Stats */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
    margin-bottom: 32px;
  }
  .stat-card {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    padding: 16px;
    text-align: center;
  }
  .stat-val {
    display: block;
    font-size: 1.25rem;
    font-weight: 800;
    margin-bottom: 4px;
    color: var(--text-1);
  }
  .stat-label {
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-3);
  }
  .lime { color: var(--lime); }

  /* Transactions */
  .section h2 {
    font-size: 1rem;
    margin-bottom: 12px;
  }
  .txn-list {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    overflow: hidden;
  }
  .txn-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    font-size: 0.875rem;
  }
  .txn-row:last-child { border-bottom: none; }
  .txn-type { color: var(--text-2); }
  .txn-amount { font-weight: 700; }
  .positive { color: var(--lime); }
  .negative { color: var(--rose); }

  @media (max-width: 640px) {
    .stats-grid { grid-template-columns: 1fr 1fr; }
    .wallet-card { flex-direction: column; align-items: flex-start; gap: 16px; }
  }
</style>
