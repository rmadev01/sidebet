<script lang="ts">
  import { onMount } from 'svelte';
  import { isAuthenticated } from '$lib/stores';
  import { getFeed, getBets, getMe } from '$lib/api';

  let feedItems = $state<any[]>([]);
  let pendingBets = $state<any[]>([]);
  let stats = $state<any>(null);
  let loading = $state(true);

  onMount(async () => {
    if (!$isAuthenticated) {
      loading = false;
      return;
    }
    try {
      const [feed, pending, me] = await Promise.all([
        getFeed().catch(() => []),
        getBets({ status: 'proposed', role: 'opponent' }).catch(() => []),
        getMe().catch(() => null),
      ]);
      feedItems = feed;
      pendingBets = pending;
      stats = me;
    } catch { /* ignore */ }
    loading = false;
  });

  function formatCoins(n: number): string {
    return n?.toLocaleString() ?? '0';
  }

  function timeAgo(dateStr: string): string {
    const diff = Date.now() - new Date(dateStr).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    return `${Math.floor(hrs / 24)}d ago`;
  }
</script>

<svelte:head>
  <title>SideBet — Dashboard</title>
  <meta name="description" content="Your peer-to-peer betting dashboard." />
</svelte:head>

<div class="dash">
  <!-- Hero -->
  <section class="hero animate-in">
    <div class="hero-glow"></div>
    {#if stats}
      <h1 class="hero-title">You're <span class="lime">{stats.wins}–{stats.losses}</span> this month</h1>
      <div class="hero-stats">
        <span><strong class="mono">{formatCoins(stats.coin_balance)}</strong> coins</span>
        <span class="sep">·</span>
        <span><strong class="mono">{formatCoins(stats.total_wagered)}</strong> wagered</span>
        <span class="sep">·</span>
        <span class="streak">{stats.wins > stats.losses ? `${stats.wins - stats.losses}W` : '—'}</span>
      </div>
    {:else}
      <h1 class="hero-title">Welcome to <span class="lime">SideBet</span></h1>
      <div class="hero-stats">
        <span>Sign in to start betting with friends</span>
      </div>
    {/if}
  </section>

  <!-- Incoming Bets -->
  {#if pendingBets.length > 0}
    <section class="section animate-in" style="animation-delay: 80ms">
      <div class="section-head">
        <h2>
          <span class="dot dot--live"></span>
          Incoming
        </h2>
        <span class="count">{pendingBets.length}</span>
      </div>

      <div class="stagger">
        {#each pendingBets as bet}
          <a href="/bets/{bet.id}" class="bet-row accent-bar accent-bar--amber">
            <div class="bet-row-body">
              <span class="bet-q">{bet.question}</span>
              <span class="bet-sub">{bet.odds_numerator}:{bet.odds_denominator} · expires {new Date(bet.expires_at).toLocaleDateString()}</span>
            </div>
            <div class="bet-row-right">
              <span class="mono amt">{formatCoins(bet.amount)} coins</span>
            </div>
          </a>
        {/each}
      </div>
    </section>
  {/if}

  <!-- Feed -->
  <section class="section animate-in" style="animation-delay: 160ms">
    <div class="section-head">
      <h2>Activity</h2>
    </div>

    {#if feedItems.length === 0 && !loading}
      <p class="empty-state">No activity yet. Add friends and place bets to see your feed!</p>
    {/if}

    <div class="feed stagger">
      {#each feedItems as item}
        <div class="feed-row accent-bar"
          class:accent-bar--lime={item.payload?.outcome?.includes('win')}
          class:accent-bar--rose={item.payload?.outcome?.includes('lose') || item.payload?.outcome?.includes('loss')}
          class:accent-bar--sky={!item.payload?.outcome}
        >
          <div class="feed-body">
            <span class="feed-q">{item.payload?.question || 'Bet update'}</span>
            <span class="feed-sub">
              {item.type} · {timeAgo(item.created_at)}
            </span>
          </div>
          <div class="feed-right">
            <span class="mono amt">{formatCoins(item.payload?.amount || 0)} coins</span>
            {#if item.payload?.status === 'settled'}
              <span class="tag tag--win">Settled</span>
            {:else}
              <span class="tag tag--active">Active</span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </section>
</div>

<style>
  .dash { max-width: 760px; }

  .empty-state {
    color: var(--text-3);
    font-size: 0.875rem;
    padding: 20px 0;
  }

  /* ── Hero ── */
  .hero {
    position: relative;
    padding: 48px 0 40px;
    margin-bottom: 12px;
  }
  .hero-glow {
    position: absolute;
    top: -40px;
    left: 50%;
    transform: translateX(-50%);
    width: 400px;
    height: 200px;
    background: radial-gradient(ellipse, rgba(200,255,0,0.06) 0%, transparent 70%);
    pointer-events: none;
  }
  .hero-title {
    font-size: 2.25rem;
    font-weight: 700;
    margin-bottom: 12px;
    position: relative;
  }
  .lime { color: var(--lime); }
  .hero-stats {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 0.875rem;
    color: var(--text-2);
    flex-wrap: wrap;
  }
  .sep { color: var(--text-3); }
  .streak {
    background: var(--amber-dim);
    color: var(--amber);
    padding: 2px 10px;
    border-radius: var(--r-full);
    font-size: 0.75rem;
    font-weight: 600;
    font-family: var(--font-display);
  }

  /* ── Sections ── */
  .section { margin-bottom: 40px; }
  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
  }
  .section-head h2 {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 1rem;
  }
  .count {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--text-3);
    background: var(--bg-raised);
    padding: 2px 8px;
    border-radius: var(--r-full);
  }

  /* ── Bet Rows ── */
  .bet-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 14px 14px 14px 18px;
    border-bottom: 1px solid var(--border);
    text-decoration: none;
    color: inherit;
    transition: background var(--dur-fast) var(--ease-out);
  }
  .bet-row:last-child { border-bottom: none; }
  .bet-row:hover { background: var(--bg-raised); }

  .bet-row-body { flex: 1; min-width: 0; }
  .bet-q {
    display: block;
    font-weight: 600;
    font-size: 0.9375rem;
    line-height: 1.35;
    margin-bottom: 3px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .bet-sub {
    font-size: 0.75rem;
    color: var(--text-3);
  }

  .bet-row-right {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
  }

  .amt {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-1);
  }

  /* ── Feed ── */
  .feed-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 12px 12px 18px;
    border-bottom: 1px solid var(--border);
  }
  .feed-row:last-child { border-bottom: none; }

  .feed-body { flex: 1; min-width: 0; }
  .feed-q {
    display: block;
    font-size: 0.875rem;
    font-weight: 500;
    line-height: 1.35;
    color: var(--text-1);
    margin-bottom: 2px;
  }
  .feed-sub { font-size: 0.75rem; color: var(--text-3); }

  .feed-right {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }

  /* ── Tags ── */
  .tag {
    font-family: var(--font-display);
    font-size: 0.6875rem;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: var(--r-sm);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .tag--win { background: var(--lime-dim); color: var(--lime); }
  .tag--loss { background: var(--rose-dim); color: var(--rose); }
  .tag--active { background: var(--sky-dim); color: var(--sky); }

  @media (max-width: 640px) {
    .hero-title { font-size: 1.625rem; }
    .bet-row, .feed-row { flex-direction: column; align-items: flex-start; gap: 8px; }
    .bet-row-right, .feed-right { width: 100%; justify-content: space-between; }
  }
</style>
