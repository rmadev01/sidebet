<script lang="ts">
  import { onMount } from 'svelte';
  import { runWhenAuthResolved } from '$lib/auth';
  import { getBets } from '$lib/api';
  import { Plus } from 'lucide-svelte';

  let activeBets = $state<any[]>([]);
  let proposedBets = $state<any[]>([]);
  let openBets = $state<any[]>([]);
  let historyBets = $state<any[]>([]);
  let tab = $state('active');
  let loading = $state(true);

  onMount(() => {
    return runWhenAuthResolved(async () => {
      loading = true;
      try {
        const [active, proposed, open, history] = await Promise.all([
          getBets({ status: 'active' }).catch(() => []),
          getBets({ status: 'proposed' }).catch(() => []),
          getBets({ status: 'open' }).catch(() => []),
          getBets({ status: 'settled' }).catch(() => [])
        ]);
        activeBets = active;
        proposedBets = proposed;
        openBets = open;
        historyBets = history;
      } catch {
        activeBets = [];
        proposedBets = [];
        openBets = [];
        historyBets = [];
      } finally {
        loading = false;
      }
    }, async () => {
      activeBets = [];
      proposedBets = [];
      openBets = [];
      historyBets = [];
      loading = false;
    });
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
    if (tab === 'open') return openBets;
    return historyBets;
  }

  const tabs = [
    { id: 'active', label: 'Active', count: () => activeBets.length },
    { id: 'proposed', label: 'Proposed', count: () => proposedBets.length },
    { id: 'open', label: 'Open', count: () => openBets.length },
    { id: 'history', label: 'History', count: () => historyBets.length },
  ];
</script>

<svelte:head>
  <title>My Bets — SideBet</title>
</svelte:head>

<div class="max-w-5xl">
  <!-- Page Header -->
  <div class="flex items-center justify-between mb-6">
    <h1 class="text-2xl font-display tracking-tight">My Bets</h1>
    <a href="/bets/new" class="inline-flex items-center justify-center gap-1.5 px-3.5 py-2 font-display text-xs font-semibold bg-lime text-white rounded-md no-underline hover:bg-lime-hover active:scale-95 transition-all duration-150">
      <Plus size={14} strokeWidth={2.5} />
      New Bet
    </a>
  </div>

  <!-- Tabs -->
  <div class="flex gap-0.5 mb-6 bg-raised/50 rounded-md p-0.5">
    {#each tabs as t}
      <button
        class="flex-1 py-2 text-[0.8125rem] font-semibold rounded-sm flex items-center justify-center gap-1.5 transition-all duration-150 cursor-pointer min-h-11 border-none
          {tab === t.id
            ? 'bg-surface text-text-1 shadow-sm'
            : 'bg-transparent text-text-3 hover:text-text-2'}"
        onclick={() => tab = t.id}
      >
        {t.label}
        <span class="font-mono text-[0.6875rem] opacity-60">{t.count()}</span>
      </button>
    {/each}
  </div>

  <!-- List -->
  {#if loading}
    <p class="text-text-3 text-sm py-8 text-center">Loading…</p>
  {:else if currentBets().length === 0}
    <p class="text-text-3 text-sm py-8 text-center">No bets here yet.</p>
  {:else}
    <div class="stagger">
      {#each currentBets() as bet}
        <a href="/bets/{bet.id}" class="group flex items-center max-sm:flex-col max-sm:items-start justify-between gap-4 max-sm:gap-2 py-3.5 px-3.5 pl-5 border-b border-border no-underline text-inherit hover:bg-raised/50 transition-all duration-150 last:border-b-0 relative before:content-[''] before:absolute before:left-0 before:top-1 before:bottom-1 before:w-[3px] before:rounded-sm
          {bet.status === 'settled' && bet.outcome?.includes('creator') ? 'before:bg-lime' : bet.status === 'settled' && bet.outcome?.includes('opponent') ? 'before:bg-rose' : bet.status === 'active' || bet.status === 'open' ? 'before:bg-sky' : 'before:bg-amber'}">
          <div class="flex-1 min-w-0">
            <span class="block font-semibold text-[0.9375rem] mb-0.5 truncate">{bet.question}</span>
            <span class="text-xs text-text-3">{bet.odds_numerator}:{bet.odds_denominator} · {timeAgo(bet.created_at)}</span>
          </div>
          <div class="flex items-center gap-3 shrink-0 max-sm:w-full max-sm:justify-between">
            <span class="font-mono tabular-nums text-sm font-semibold">{formatCoins(bet.amount)} coins</span>
            <span class="text-[0.6875rem] font-bold uppercase tracking-wide px-2 py-0.5 rounded-sm
              {bet.outcome?.includes('win') ? 'bg-lime-dim text-lime' : bet.outcome?.includes('lose') ? 'bg-rose-dim text-rose' : bet.status === 'active' ? 'bg-sky-dim text-sky' : bet.status === 'proposed' ? 'bg-amber-dim text-amber' : 'bg-sky-dim text-sky'}">{bet.status}</span>
          </div>
        </a>
      {/each}
    </div>
  {/if}
</div>
