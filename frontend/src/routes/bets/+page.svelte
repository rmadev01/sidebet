<script lang="ts">
  let activeTab = $state<'active' | 'proposed' | 'history'>('active');

  const bets = {
    active: [
      { id: 'b1', question: 'Lakers defeat Celtics on Mar 5', position: 'Lakers win', amount: 0.5, odds: '3:2', opponent: 'chad_bets' },
      { id: 'b2', question: 'Bitcoin hits $150k by June 2026', position: 'Yes', amount: 1.0, odds: '2:1', opponent: 'crypto_mike' },
    ],
    proposed: [
      { id: 'b3', question: 'Knicks beat 76ers tonight', position: 'Knicks win', amount: 0.1, odds: '1:1', opponent: 'hoops_fan', expires: '23h left' },
    ],
    history: [
      { id: 'b4', question: 'Warriors win 2025-26 title', position: 'Yes', amount: 0.25, odds: '5:1', opponent: 'dub_nation', outcome: 'win' },
      { id: 'b5', question: 'Fed cuts rates in Jan 2026', position: 'Yes', amount: 0.5, odds: '1:1', opponent: 'macro_trader', outcome: 'loss' },
    ],
  };

  function accentClass(tab: string, outcome?: string) {
    if (tab === 'active') return 'accent-bar--sky';
    if (tab === 'proposed') return 'accent-bar--amber';
    if (outcome === 'win') return 'accent-bar--lime';
    return 'accent-bar--rose';
  }

  function statusText(tab: string, outcome?: string) {
    if (tab === 'active') return 'Active';
    if (tab === 'proposed') return 'Pending';
    if (outcome === 'win') return 'Won';
    return 'Lost';
  }

  function tagClass(tab: string, outcome?: string) {
    if (tab === 'active') return 'tag--active';
    if (tab === 'proposed') return 'tag--pending';
    if (outcome === 'win') return 'tag--win';
    return 'tag--loss';
  }
</script>

<svelte:head>
  <title>SideBet — My Bets</title>
</svelte:head>

<div class="bets-page">
  <div class="page-head animate-in">
    <div>
      <h1>My Bets</h1>
      <p class="subtitle">Track all your active and past bets</p>
    </div>
    <a href="/bets/new" class="btn btn-primary">New Bet</a>
  </div>

  <div class="seg-control animate-in" style="animation-delay:60ms">
    <button class="seg-btn" class:active={activeTab === 'active'} onclick={() => activeTab = 'active'}>Active {bets.active.length}</button>
    <button class="seg-btn" class:active={activeTab === 'proposed'} onclick={() => activeTab = 'proposed'}>Proposed {bets.proposed.length}</button>
    <button class="seg-btn" class:active={activeTab === 'history'} onclick={() => activeTab = 'history'}>History {bets.history.length}</button>
  </div>

  <div class="stagger">
    {#each bets[activeTab] as bet}
      <a href="/bets/{bet.id}" class="bet-row accent-bar {accentClass(activeTab, bet.outcome)}">
        <div class="bet-body">
          <span class="bet-q">{bet.question}</span>
          <span class="bet-sub">
            vs @{bet.opponent} · Your pick: {bet.position} · {bet.odds}
            {#if bet.expires} · {bet.expires}{/if}
          </span>
        </div>
        <div class="bet-right">
          <span class="mono amt">{bet.amount} ETH</span>
          <span class="tag {tagClass(activeTab, bet.outcome)}">{statusText(activeTab, bet.outcome)}</span>
        </div>
      </a>
    {/each}
  </div>
</div>

<style>
  .bets-page { max-width: 760px; }

  .page-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 12px;
    gap: 16px;
  }
  .page-head h1 { margin-bottom: 4px; }
  .subtitle { color: var(--text-2); font-size: 0.9375rem; }

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

  .bet-body { flex: 1; min-width: 0; }
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
  .bet-sub { font-size: 0.75rem; color: var(--text-3); }

  .bet-right {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }

  .amt { font-size: 0.875rem; font-weight: 600; color: var(--text-1); }

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
  .tag--pending { background: var(--amber-dim); color: var(--amber); }

  @media (max-width: 640px) {
    .page-head { flex-direction: column; }
    .bet-row { flex-direction: column; align-items: flex-start; gap: 8px; }
    .bet-right { width: 100%; justify-content: space-between; }
  }
</style>
