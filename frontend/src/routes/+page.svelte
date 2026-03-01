<script lang="ts">
  import { onMount } from 'svelte';

  let feedItems = $state<any[]>([]);
  let pendingBets = $state<any[]>([]);

  const demoFeed = [
    { id: '1', type: 'settled', question: 'Lakers defeat Celtics on Mar 5', from: 'chad_bets', amount: 0.5, outcome: 'win', time: '12m ago' },
    { id: '2', type: 'active', question: 'Will Bitcoin hit $150k by June 2026?', from: 'crypto_mike', amount: 1.0, outcome: null, time: '1h ago' },
    { id: '3', type: 'settled', question: 'Nuggets win Western Conference', from: 'dub_nation', amount: 0.25, outcome: 'loss', time: '2h ago' },
    { id: '4', type: 'settled', question: 'Fed raises rates in Q1', from: 'macro_trader', amount: 0.75, outcome: 'win', time: '5h ago' },
  ];

  const demoPending = [
    { id: 'p1', question: 'Knicks beat 76ers tonight', from: 'hoops_fan', amount: 0.1, odds: '3:2', expires: '23h left' },
    { id: 'p2', question: 'Trump wins 2028 GOP primary', from: 'poly_whale', amount: 0.5, odds: '1:1', expires: '2d left' },
  ];

  onMount(() => {
    feedItems = demoFeed;
    pendingBets = demoPending;
  });
</script>

<svelte:head>
  <title>SideBet — Dashboard</title>
  <meta name="description" content="Your peer-to-peer betting dashboard." />
</svelte:head>

<div class="dash">
  <!-- Hero -->
  <section class="hero animate-in">
    <div class="hero-glow"></div>
    <h1 class="hero-title">You're <span class="lime">12–8</span> this month</h1>
    <div class="hero-stats">
      <span><strong class="mono">2.45 ETH</strong> wagered</span>
      <span class="sep">·</span>
      <span><strong class="mono">3</strong> active bets</span>
      <span class="sep">·</span>
      <span class="streak">3W streak</span>
    </div>
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
              <span class="bet-sub">from @{bet.from} · {bet.odds} · {bet.expires}</span>
            </div>
            <div class="bet-row-right">
              <span class="mono amt">{bet.amount} ETH</span>
              <div class="bet-actions">
                <button class="btn btn-accept btn-sm" onclick={(e) => e.stopPropagation()}>Accept</button>
                <button class="btn btn-ghost btn-sm" onclick={(e) => e.stopPropagation()}>Decline</button>
              </div>
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

    <div class="feed stagger">
      {#each feedItems as item}
        <div class="feed-row accent-bar"
          class:accent-bar--lime={item.outcome === 'win'}
          class:accent-bar--rose={item.outcome === 'loss'}
          class:accent-bar--sky={item.outcome === null}
        >
          <div class="feed-body">
            <span class="feed-q">{item.question}</span>
            <span class="feed-sub">
              vs @{item.from} · {item.time}
            </span>
          </div>
          <div class="feed-right">
            <span class="mono amt">{item.amount} ETH</span>
            {#if item.outcome === 'win'}
              <span class="tag tag--win">Won</span>
            {:else if item.outcome === 'loss'}
              <span class="tag tag--loss">Lost</span>
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

  .bet-actions {
    display: flex;
    gap: 6px;
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
