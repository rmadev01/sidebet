<script lang="ts">
  import { onMount } from 'svelte';
  import { isAuthenticated, user } from '$lib/stores';

  let feedItems = $state<any[]>([]);
  let pendingBets = $state<any[]>([]);
  let loading = $state(true);

  // Demo data for visual preview
  const demoFeed = [
    { id: '1', item_type: 'bet_settled', payload: { question: 'Lakers defeat Celtics on Mar 5', creator_id: 'demo', amount_wei: 500000000000000000, outcome: 'creator_wins', status: 'settled' }, created_at: new Date().toISOString() },
    { id: '2', item_type: 'bet_active', payload: { question: 'Will Bitcoin hit $150k by June 2026?', creator_id: 'demo', amount_wei: 1000000000000000000, status: 'active' }, created_at: new Date(Date.now() - 3600000).toISOString() },
    { id: '3', item_type: 'bet_settled', payload: { question: 'Nuggets win Western Conference', creator_id: 'demo', amount_wei: 250000000000000000, outcome: 'opponent_wins', status: 'settled' }, created_at: new Date(Date.now() - 7200000).toISOString() },
  ];

  const demoPending = [
    { id: 'p1', question: 'Knicks beat 76ers tonight', creator_position: 'Knicks win', amount_wei: 100000000000000000, odds_numerator: 3, odds_denominator: 2, creator_id: 'friend1', expires_at: new Date(Date.now() + 86400000).toISOString() },
    { id: 'p2', question: 'Trump wins 2028 GOP primary', creator_position: 'Yes', amount_wei: 500000000000000000, odds_numerator: 1, odds_denominator: 1, creator_id: 'friend2', expires_at: new Date(Date.now() + 172800000).toISOString() },
  ];

  onMount(() => {
    feedItems = demoFeed;
    pendingBets = demoPending;
    loading = false;
  });

  function formatWei(wei: number): string {
    return (wei / 1e18).toFixed(3) + ' ETH';
  }

  function timeAgo(dateStr: string): string {
    const diff = Date.now() - new Date(dateStr).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    return `${Math.floor(hours / 24)}d ago`;
  }

  function countdown(dateStr: string): string {
    const diff = new Date(dateStr).getTime() - Date.now();
    if (diff <= 0) return 'Expired';
    const hours = Math.floor(diff / 3600000);
    const mins = Math.floor((diff % 3600000) / 60000);
    return `${hours}h ${mins}m left`;
  }
</script>

<svelte:head>
  <title>SideBet — Dashboard</title>
  <meta name="description" content="Your peer-to-peer betting dashboard. Track bets, view activity, and challenge friends." />
</svelte:head>

<div class="dashboard">
  <div class="page-header animate-slide-up">
    <div>
      <h1>Dashboard</h1>
      <p class="text-secondary">Your betting activity at a glance</p>
    </div>
    <a href="/bets/new" class="btn btn-primary">
      ➕ New Bet
    </a>
  </div>

  <!-- Stats Row -->
  <div class="stats-row animate-slide-up" style="animation-delay: 80ms">
    <div class="stat-card">
      <div class="stat-value" style="color: var(--accent-green)">12</div>
      <div class="stat-label">Wins</div>
    </div>
    <div class="stat-card">
      <div class="stat-value" style="color: var(--accent-red)">8</div>
      <div class="stat-label">Losses</div>
    </div>
    <div class="stat-card">
      <div class="stat-value" style="color: var(--accent-gold)">3</div>
      <div class="stat-label">Active</div>
    </div>
    <div class="stat-card">
      <div class="stat-value" style="color: var(--accent-blue)">2.45 ETH</div>
      <div class="stat-label">Total Wagered</div>
    </div>
  </div>

  <div class="dashboard-grid">
    <!-- Incoming Bets -->
    <section class="animate-slide-up" style="animation-delay: 160ms">
      <h3 class="section-title">
        <span class="status-dot live"></span>
        Incoming Bets
      </h3>

      {#if pendingBets.length === 0}
        <div class="empty-state">
          <div class="icon">📭</div>
          <p>No pending bets</p>
        </div>
      {:else}
        <div class="bet-list stagger">
          {#each pendingBets as bet}
            <div class="card bet-card-incoming">
              <div class="bet-card-top">
                <span class="badge badge-gold">Pending</span>
                <span class="text-xs text-muted">{countdown(bet.expires_at)}</span>
              </div>
              <p class="bet-question">{bet.question}</p>
              <div class="bet-meta">
                <span class="bet-amount">{formatWei(bet.amount_wei)}</span>
                <span class="text-muted">·</span>
                <span class="text-secondary">Odds {bet.odds_numerator}:{bet.odds_denominator}</span>
              </div>
              <div class="bet-actions">
                <button class="btn btn-success btn-sm">Accept</button>
                <button class="btn btn-ghost btn-sm">Decline</button>
                <a href="/bets/{bet.id}" class="btn btn-ghost btn-sm">Details →</a>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- Activity Feed -->
    <section class="animate-slide-up" style="animation-delay: 240ms">
      <h3 class="section-title">Activity Feed</h3>

      {#if feedItems.length === 0}
        <div class="empty-state">
          <div class="icon">📋</div>
          <p>No recent activity</p>
        </div>
      {:else}
        <div class="feed-list stagger">
          {#each feedItems as item}
            <div class="card feed-card">
              <div class="feed-card-header">
                <div class="avatar avatar-sm">
                  {item.payload.creator_id?.[0]?.toUpperCase() || '?'}
                </div>
                <div class="feed-card-info">
                  <span class="feed-type">
                    {#if item.item_type === 'bet_settled'}
                      <span class="badge" class:badge-green={item.payload.outcome === 'creator_wins'} class:badge-red={item.payload.outcome === 'opponent_wins'}>
                        {item.payload.outcome === 'creator_wins' ? 'Won' : 'Lost'}
                      </span>
                    {:else}
                      <span class="badge badge-blue">Active</span>
                    {/if}
                  </span>
                  <span class="text-xs text-muted">{timeAgo(item.created_at)}</span>
                </div>
              </div>
              <p class="feed-question">{item.payload.question}</p>
              <span class="feed-amount">{formatWei(item.payload.amount_wei)}</span>
            </div>
          {/each}
        </div>
      {/if}
    </section>
  </div>
</div>

<style>
  .dashboard { max-width: 1100px; }

  .page-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: var(--space-xl);
    gap: var(--space-md);
  }
  .page-header h1 { margin-bottom: 4px; }

  /* Stats */
  .stats-row {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-md);
    margin-bottom: var(--space-2xl);
  }
  .stat-card {
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    padding: var(--space-lg);
    text-align: center;
  }
  .stat-value {
    font-size: 1.75rem;
    font-weight: 800;
    letter-spacing: -0.03em;
    line-height: 1;
    margin-bottom: var(--space-xs);
  }
  .stat-label {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
  }

  /* Grid */
  .dashboard-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-xl);
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-md);
    font-size: 1rem;
    font-weight: 700;
  }

  /* Bet incoming cards */
  .bet-list { display: flex; flex-direction: column; gap: var(--space-md); }

  .bet-card-incoming { position: relative; }
  .bet-card-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-sm);
  }
  .bet-question {
    font-weight: 600;
    font-size: 0.9375rem;
    margin-bottom: var(--space-sm);
    line-height: 1.4;
  }
  .bet-meta {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: 0.875rem;
    margin-bottom: var(--space-md);
  }
  .bet-amount {
    font-weight: 700;
    color: var(--accent-blue);
    font-family: var(--font-mono);
  }
  .bet-actions {
    display: flex;
    gap: var(--space-sm);
  }

  /* Feed */
  .feed-list { display: flex; flex-direction: column; gap: var(--space-sm); }

  .feed-card { padding: var(--space-md); }
  .feed-card-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
  }
  .feed-card-info {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex: 1;
  }
  .feed-question {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: 4px;
  }
  .feed-amount {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--accent-blue);
    font-family: var(--font-mono);
  }

  @media (max-width: 768px) {
    .stats-row { grid-template-columns: repeat(2, 1fr); }
    .dashboard-grid { grid-template-columns: 1fr; }
    .page-header { flex-direction: column; }
  }
</style>
