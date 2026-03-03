<script lang="ts">
  import { onMount } from 'svelte';
  import { isAuthenticated } from '$lib/stores';
  import { getBets } from '$lib/api';

  let activeBets = $state<any[]>([]);
  let proposedBets = $state<any[]>([]);
  let historyBets = $state<any[]>([]);
  let tab = $state('active');
  let loading = $state(true);

  onMount(async () => {
    if (!$isAuthenticated) { loading = false; return; }
    try {
      const [active, proposed, history] = await Promise.all([
        getBets({ status: 'active' }).catch(() => []),
        getBets({ status: 'proposed' }).catch(() => []),
        getBets({ status: 'settled' }).catch(() => []),
      ]);
      activeBets = active;
      proposedBets = proposed;
      historyBets = history;
    } catch { /* ignore */ }
    loading = false;
  });

  function formatCoins(n: number) { return n?.toLocaleString() ?? '0'; }
  function timeAgo(d: string) {
    const diff = Date.now() - new Date(d).getTime();
    const h = Math.floor(diff / 3600000);
    return h < 24 ? `${h}h ago` : `${Math.floor(h / 24)}d ago`;
  }

  function currentBets() {
    if (tab === 'active') return activeBets;
    if (tab === 'proposed') return proposedBets;
    return historyBets;
  }
</script>

<svelte:head>
  <title>My Bets — SideBet</title>
</svelte:head>

<div class="bets-page">
  <div class="page-head">
    <h1>My Bets</h1>
    <a href="/bets/new" class="btn btn-primary btn-sm">+ New Bet</a>
  </div>

  <!-- Tabs -->
  <div class="tabs">
    <button class="tab" class:active={tab === 'active'} onclick={() => tab = 'active'}>
      Active <span class="tab-count">{activeBets.length}</span>
    </button>
    <button class="tab" class:active={tab === 'proposed'} onclick={() => tab = 'proposed'}>
      Proposed <span class="tab-count">{proposedBets.length}</span>
    </button>
    <button class="tab" class:active={tab === 'history'} onclick={() => tab = 'history'}>
      History <span class="tab-count">{historyBets.length}</span>
    </button>
  </div>

  <!-- List -->
  {#if loading}
    <p class="empty-state">Loading…</p>
  {:else if currentBets().length === 0}
    <p class="empty-state">No bets here yet.</p>
  {:else}
    <div class="bet-list stagger">
      {#each currentBets() as bet}
        <a href="/bets/{bet.id}" class="bet-card accent-bar"
          class:accent-bar--lime={bet.status === 'settled' && bet.outcome?.includes('creator')}
          class:accent-bar--rose={bet.status === 'settled' && bet.outcome?.includes('opponent')}
          class:accent-bar--sky={bet.status === 'active'}
          class:accent-bar--amber={bet.status === 'proposed'}
        >
          <div class="bet-card-body">
            <span class="bet-q">{bet.question}</span>
            <span class="bet-meta">{bet.odds_numerator}:{bet.odds_denominator} · {timeAgo(bet.created_at)}</span>
          </div>
          <div class="bet-card-right">
            <span class="mono">{formatCoins(bet.amount)} coins</span>
            <span class="tag"
              class:tag--win={bet.outcome?.includes('win')}
              class:tag--loss={bet.outcome?.includes('lose')}
              class:tag--active={bet.status === 'active'}
              class:tag--pending={bet.status === 'proposed'}
            >{bet.status}</span>
          </div>
        </a>
      {/each}
    </div>
  {/if}
</div>

<style>
  .bets-page { max-width: 760px; }

  .page-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 24px;
  }
  .page-head h1 { font-size: 1.5rem; }

  .tabs {
    display: flex;
    gap: 2px;
    margin-bottom: 24px;
    background: var(--bg-sunken);
    border-radius: var(--r-md);
    padding: 2px;
  }
  .tab {
    flex: 1;
    padding: 8px 0;
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--text-3);
    background: none;
    border: none;
    border-radius: var(--r-sm);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    transition: all var(--dur-fast) var(--ease-out);
  }
  .tab:hover { color: var(--text-2); }
  .tab.active { background: var(--bg-surface); color: var(--text-1); }
  .tab-count {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    opacity: 0.6;
  }

  .empty-state {
    color: var(--text-3);
    font-size: 0.875rem;
    padding: 32px 0;
    text-align: center;
  }

  .bet-card {
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
  .bet-card:last-child { border-bottom: none; }
  .bet-card:hover { background: var(--bg-raised); }

  .bet-card-body { flex: 1; min-width: 0; }
  .bet-q {
    display: block;
    font-weight: 600;
    font-size: 0.9375rem;
    margin-bottom: 2px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .bet-meta { font-size: 0.75rem; color: var(--text-3); }

  .bet-card-right {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
    font-size: 0.875rem;
    font-weight: 600;
  }

  .tag {
    font-size: 0.6875rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 2px 8px;
    border-radius: var(--r-sm);
  }
  .tag--win { background: var(--lime-dim); color: var(--lime); }
  .tag--loss { background: var(--rose-dim); color: var(--rose); }
  .tag--active { background: var(--sky-dim); color: var(--sky); }
  .tag--pending { background: var(--amber-dim); color: var(--amber); }

  @media (max-width: 640px) {
    .bet-card { flex-direction: column; align-items: flex-start; gap: 8px; }
    .bet-card-right { width: 100%; justify-content: space-between; }
  }
</style>
