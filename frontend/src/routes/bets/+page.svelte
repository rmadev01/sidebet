<script lang="ts">
  let activeTab = $state<'active' | 'proposed' | 'history'>('active');

  const demoBets = {
    active: [
      { id: 'b1', question: 'Lakers defeat Celtics on Mar 5', creator_position: 'Lakers win', opponent_position: 'Celtics win', amount_wei: 500000000000000000, odds_numerator: 3, odds_denominator: 2, status: 'active', opponent_name: 'chad_bets', created_at: '2026-03-01T10:00:00Z' },
      { id: 'b2', question: 'Bitcoin hits $150k by June 2026', creator_position: 'Yes', opponent_position: 'No', amount_wei: 1000000000000000000, odds_numerator: 2, odds_denominator: 1, status: 'active', opponent_name: 'crypto_mike', created_at: '2026-02-28T15:00:00Z' },
    ],
    proposed: [
      { id: 'b3', question: 'Knicks beat 76ers tonight', creator_position: 'Knicks win', opponent_position: '76ers win', amount_wei: 100000000000000000, odds_numerator: 1, odds_denominator: 1, status: 'proposed', opponent_name: 'hoops_fan', created_at: '2026-03-01T12:00:00Z', expires_at: '2026-03-02T12:00:00Z' },
    ],
    history: [
      { id: 'b4', question: 'Warriors win 2025-26 title', creator_position: 'Yes', opponent_position: 'No', amount_wei: 250000000000000000, odds_numerator: 5, odds_denominator: 1, status: 'settled', opponent_name: 'dub_nation', outcome: 'creator_wins', created_at: '2025-10-15T10:00:00Z' },
      { id: 'b5', question: 'Fed cuts rates in Jan 2026', creator_position: 'Yes', opponent_position: 'No', amount_wei: 500000000000000000, odds_numerator: 1, odds_denominator: 1, status: 'settled', opponent_name: 'macro_trader', outcome: 'opponent_wins', created_at: '2025-12-01T10:00:00Z' },
    ],
  };

  function formatWei(wei: number): string {
    return (wei / 1e18).toFixed(3) + ' ETH';
  }

  function statusBadgeClass(status: string, outcome?: string): string {
    if (status === 'settled' && outcome === 'creator_wins') return 'badge-green';
    if (status === 'settled' && outcome === 'opponent_wins') return 'badge-red';
    if (status === 'active') return 'badge-blue';
    if (status === 'proposed') return 'badge-gold';
    return 'badge-blue';
  }

  function statusLabel(status: string, outcome?: string): string {
    if (status === 'settled' && outcome === 'creator_wins') return 'Won';
    if (status === 'settled' && outcome === 'opponent_wins') return 'Lost';
    return status.charAt(0).toUpperCase() + status.slice(1);
  }
</script>

<svelte:head>
  <title>SideBet — My Bets</title>
</svelte:head>

<div class="bets-page">
  <div class="page-header animate-slide-up">
    <div>
      <h1>My Bets</h1>
      <p class="text-secondary">Track all your active and past bets</p>
    </div>
    <a href="/bets/new" class="btn btn-primary">➕ New Bet</a>
  </div>

  <div class="tabs animate-slide-up" style="animation-delay: 60ms">
    <button class="tab" class:active={activeTab === 'active'} onclick={() => activeTab = 'active'}>
      Active ({demoBets.active.length})
    </button>
    <button class="tab" class:active={activeTab === 'proposed'} onclick={() => activeTab = 'proposed'}>
      Proposed ({demoBets.proposed.length})
    </button>
    <button class="tab" class:active={activeTab === 'history'} onclick={() => activeTab = 'history'}>
      History ({demoBets.history.length})
    </button>
  </div>

  <div class="stagger">
    {#each demoBets[activeTab] as bet}
      <a href="/bets/{bet.id}" class="card bet-row">
        <div class="bet-row-left">
          <div class="avatar avatar-sm">{bet.opponent_name[0].toUpperCase()}</div>
          <div class="bet-row-info">
            <span class="bet-row-question">{bet.question}</span>
            <span class="text-xs text-muted">vs @{bet.opponent_name} · Your pick: {bet.creator_position}</span>
          </div>
        </div>
        <div class="bet-row-right">
          <span class="bet-row-amount">{formatWei(bet.amount_wei)}</span>
          <span class="badge {statusBadgeClass(bet.status, bet.outcome)}">{statusLabel(bet.status, bet.outcome)}</span>
        </div>
      </a>
    {/each}
  </div>
</div>

<style>
  .bets-page { max-width: 900px; }
  .page-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: var(--space-lg);
  }
  .page-header h1 { margin-bottom: 4px; }

  .bet-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md) var(--space-lg);
    margin-bottom: var(--space-sm);
    text-decoration: none;
    color: inherit;
    gap: var(--space-md);
  }

  .bet-row-left {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    flex: 1;
    min-width: 0;
  }

  .bet-row-info {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .bet-row-question {
    font-weight: 600;
    font-size: 0.9375rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .bet-row-right {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    flex-shrink: 0;
  }

  .bet-row-amount {
    font-weight: 700;
    font-family: var(--font-mono);
    color: var(--accent-blue);
    font-size: 0.9375rem;
  }

  @media (max-width: 640px) {
    .bet-row { flex-direction: column; align-items: flex-start; }
    .bet-row-right { margin-top: var(--space-sm); }
  }
</style>
