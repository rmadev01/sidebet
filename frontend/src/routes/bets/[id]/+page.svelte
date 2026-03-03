<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { isAuthenticated, user, coinBalance } from '$lib/stores';
  import { getBet, acceptBet, declineBet, cancelBet, settleBet, getUser } from '$lib/api';

  let bet = $state<any>(null);
  let creator = $state<any>(null);
  let opponent = $state<any>(null);
  let loading = $state(true);
  let acting = $state(false);
  let error = $state('');

  $effect(() => {
    load();
  });

  async function load() {
    loading = true;
    try {
      bet = await getBet($page.params.id);
      // Fetch user details in parallel — these are UUIDs so use the bet data
    } catch (e: any) {
      error = e.message;
    }
    loading = false;
  }

  function formatCoins(n: number) { return n?.toLocaleString() ?? '0'; }
  function isCreator() { return bet?.creator_id === $user?.id; }
  function isOpponent() { return bet?.opponent_id === $user?.id; }

  async function doAccept() {
    acting = true;
    error = '';
    try {
      bet = await acceptBet(bet.id);
      coinBalance.update(b => b - bet.amount);
    } catch (e: any) { error = e.message; }
    acting = false;
  }

  async function doDecline() {
    acting = true;
    error = '';
    try {
      await declineBet(bet.id);
      bet.status = 'declined';
    } catch (e: any) { error = e.message; }
    acting = false;
  }

  async function doCancel() {
    acting = true;
    error = '';
    try {
      await cancelBet(bet.id);
      bet.status = 'cancelled';
      coinBalance.update(b => b + bet.amount);
    } catch (e: any) { error = e.message; }
    acting = false;
  }

  async function doSettle(winner: string) {
    acting = true;
    error = '';
    try {
      bet = await settleBet(bet.id, winner);
    } catch (e: any) { error = e.message; }
    acting = false;
  }
</script>

<svelte:head>
  <title>Bet Details — SideBet</title>
</svelte:head>

<div class="bet-detail">
  {#if loading}
    <p class="empty-state">Loading…</p>
  {:else if !bet}
    <p class="empty-state">Bet not found.</p>
  {:else}
    <a href="/bets" class="back-link">← All Bets</a>

    <div class="detail-card">
      <!-- Status Badge -->
      <div class="status-row">
        <span class="status-badge"
          class:status--active={bet.status === 'active'}
          class:status--proposed={bet.status === 'proposed'}
          class:status--settled={bet.status === 'settled'}
          class:status--cancelled={bet.status === 'cancelled' || bet.status === 'declined'}
        >{bet.status.toUpperCase()}</span>
        <span class="mono text-3">{new Date(bet.created_at).toLocaleDateString()}</span>
      </div>

      <h1 class="bet-question">{bet.question}</h1>

      <!-- Positions -->
      <div class="positions">
        <div class="pos-card" class:winner={bet.winner_id === bet.creator_id}>
          <span class="pos-label">Creator</span>
          <span class="pos-pick">{bet.creator_position}</span>
          {#if bet.winner_id === bet.creator_id}<span class="winner-badge">🏆 Winner</span>{/if}
        </div>
        <div class="pos-vs">vs</div>
        <div class="pos-card" class:winner={bet.winner_id === bet.opponent_id}>
          <span class="pos-label">Opponent</span>
          <span class="pos-pick">{bet.opponent_position}</span>
          {#if bet.winner_id === bet.opponent_id}<span class="winner-badge">🏆 Winner</span>{/if}
        </div>
      </div>

      <!-- Details -->
      <div class="detail-grid">
        <div class="detail-item">
          <span class="detail-label">Wager</span>
          <span class="detail-val mono">{formatCoins(bet.amount)} coins</span>
        </div>
        <div class="detail-item">
          <span class="detail-label">Odds</span>
          <span class="detail-val mono">{bet.odds_numerator}:{bet.odds_denominator}</span>
        </div>
        <div class="detail-item">
          <span class="detail-label">Expires</span>
          <span class="detail-val">{new Date(bet.expires_at).toLocaleDateString()}</span>
        </div>
        {#if bet.resolved_at}
          <div class="detail-item">
            <span class="detail-label">Settled</span>
            <span class="detail-val">{new Date(bet.resolved_at).toLocaleDateString()}</span>
          </div>
        {/if}
      </div>

      {#if error}
        <p class="error">{error}</p>
      {/if}

      <!-- Actions -->
      <div class="actions">
        {#if bet.status === 'proposed' && isOpponent()}
          <button class="btn btn-primary" onclick={doAccept} disabled={acting}>Accept</button>
          <button class="btn btn-ghost" onclick={doDecline} disabled={acting}>Decline</button>
        {/if}
        {#if bet.status === 'proposed' && isCreator()}
          <button class="btn btn-ghost" onclick={doCancel} disabled={acting}>Cancel</button>
        {/if}
        {#if bet.status === 'active' && (isCreator() || isOpponent())}
          <span class="settle-label">Settle as:</span>
          <button class="btn btn-primary" onclick={() => doSettle('creator')} disabled={acting}>Creator Wins</button>
          <button class="btn btn-ghost" onclick={() => doSettle('opponent')} disabled={acting}>Opponent Wins</button>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .bet-detail { max-width: 640px; }

  .back-link {
    display: inline-block;
    margin-bottom: 16px;
    font-size: 0.875rem;
    color: var(--text-3);
    text-decoration: none;
  }
  .back-link:hover { color: var(--text-2); }

  .detail-card {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    padding: 32px;
  }

  .status-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
  }
  .status-badge {
    font-size: 0.6875rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    padding: 4px 10px;
    border-radius: var(--r-sm);
  }
  .status--active { background: var(--sky-dim); color: var(--sky); }
  .status--proposed { background: var(--amber-dim); color: var(--amber); }
  .status--settled { background: var(--lime-dim); color: var(--lime); }
  .status--cancelled { background: var(--bg-raised); color: var(--text-3); }

  .bet-question {
    font-size: 1.375rem;
    font-weight: 700;
    margin-bottom: 24px;
    line-height: 1.3;
  }

  .positions {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 24px;
  }
  .pos-card {
    flex: 1;
    background: var(--bg-raised);
    border-radius: var(--r-md);
    padding: 16px;
    text-align: center;
    border: 2px solid transparent;
  }
  .pos-card.winner { border-color: var(--lime); }
  .pos-label {
    display: block;
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-3);
    margin-bottom: 6px;
  }
  .pos-pick {
    display: block;
    font-weight: 700;
    font-size: 1rem;
    color: var(--text-1);
  }
  .pos-vs {
    font-size: 0.75rem;
    color: var(--text-3);
    font-weight: 700;
  }
  .winner-badge {
    display: inline-block;
    margin-top: 6px;
    font-size: 0.75rem;
    color: var(--lime);
  }

  .detail-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 12px;
    margin-bottom: 24px;
    padding: 16px;
    background: var(--bg-raised);
    border-radius: var(--r-md);
  }
  .detail-item { text-align: center; }
  .detail-label {
    display: block;
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-3);
    margin-bottom: 4px;
  }
  .detail-val {
    font-weight: 700;
    font-size: 0.9375rem;
    color: var(--text-1);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }
  .settle-label {
    font-size: 0.875rem;
    color: var(--text-3);
    font-weight: 600;
  }

  .error {
    color: var(--rose);
    font-size: 0.875rem;
    margin-bottom: 12px;
  }

  @media (max-width: 640px) {
    .positions { flex-direction: column; }
    .detail-card { padding: 20px; }
  }
</style>
