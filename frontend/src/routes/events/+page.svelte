<script lang="ts">
  import { onMount } from 'svelte';
  import { authReady, isAuthenticated } from '$lib/stores';
  import { getEvents, syncEvents } from '$lib/api';
  import { RefreshCw, Calendar, Zap } from 'lucide-svelte';

  let events = $state<any[]>([]);
  let league = $state('all');
  let loading = $state(true);
  let syncing = $state(false);

  const LEAGUES = [
    { id: 'all', label: 'All' },
    { id: 'nba', label: 'NBA' },
    { id: 'nfl', label: 'NFL' },
    { id: 'mlb', label: 'MLB' },
    { id: 'nhl', label: 'NHL' },
    { id: 'politics', label: 'Politics' },
  ];

  async function loadEvents() {
    loading = true;
    try {
      const params: Record<string, string> = {};
      if (league !== 'all') params.category = league;
      const all = await getEvents(params);
      events = all.filter((e: any) => e.status !== 'ended' && e.status !== 'cancelled');
    } catch { /* fallback to empty */ }
    loading = false;
  }

  async function handleSync() {
    syncing = true;
    try {
      await syncEvents();
      await loadEvents();
    } catch { /* ignore */ }
    syncing = false;
  }

  onMount(loadEvents);

  function selectLeague(id: string) {
    league = id;
    loadEvents();
  }

  function statusBadge(status: string) {
    if (status === 'live') return { text: 'LIVE', cls: 'live' };
    if (status === 'ended' || status === 'finalized') return { text: 'FINAL', cls: 'ended' };
    if (status === 'in_progress') return { text: 'IN PROGRESS', cls: 'live' };
    return null;
  }
</script>

<svelte:head>
  <title>Events — SideBet</title>
  <meta name="description" content="Browse live sports events and odds to bet on." />
</svelte:head>

<div>
  <!-- Header -->
  <div class="flex items-center justify-between mb-6">
    <h1 class="text-2xl font-display tracking-tight">Events</h1>
    {#if $authReady && $isAuthenticated}
      <button class="inline-flex items-center justify-center gap-1.5 px-3.5 py-1.5 text-[0.8125rem] font-semibold bg-transparent text-text-2 border border-border rounded-md hover:bg-raised hover:text-text-1 hover:border-border-strong active:scale-95 transition-all duration-150 cursor-pointer" onclick={handleSync} disabled={syncing}>
        <RefreshCw size={14} class={syncing ? 'animate-spin' : ''} />
        {syncing ? 'Syncing…' : 'Sync'}
      </button>
    {/if}
  </div>

  <!-- League Tabs -->
  <div class="flex gap-1.5 mb-6 flex-wrap">
    {#each LEAGUES as l}
      <button
        class="px-4 py-1.5 text-[0.8125rem] font-semibold rounded-full cursor-pointer transition-all duration-150 min-h-11 border
          {league === l.id
            ? 'bg-lime-dim border-lime text-lime-hover'
            : 'bg-raised border-border text-text-3 hover:text-text-2 hover:border-border-strong'}"
        onclick={() => selectLeague(l.id)}
      >{l.label}</button>
    {/each}
  </div>

  {#if loading}
    <p class="text-text-3 text-sm py-10 text-center">Loading…</p>
  {:else if events.length === 0}
    <p class="text-text-3 text-sm py-10 text-center">No events available. Try syncing to pull the latest from SportsGameOdds.</p>
  {:else}
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 stagger">
      {#each events as event}
        <div class="bg-surface border border-border rounded-lg p-5 flex flex-col transition-all duration-200 hover:border-border-strong hover:shadow-sm animate-enter">
          <!-- Event Header -->
          <div class="flex items-center justify-between mb-2.5">
            <span class="text-[0.6875rem] font-bold uppercase tracking-wide text-sky">{event.league || event.category}</span>
            <div class="flex items-center gap-1.5">
              {#if statusBadge(event.status)}
                {@const badge = statusBadge(event.status)}
                <span class="text-[0.6875rem] font-bold px-2 py-0.5 rounded-sm flex items-center gap-1
                  {badge?.cls === 'live' ? 'text-rose bg-rose-dim' : 'text-text-3 bg-raised'}">
                  {#if event.status === 'live'}<span class="w-1.5 h-1.5 rounded-full bg-rose animate-pulse-live inline-block"></span>{/if}
                  {badge?.text}
                </span>
              {:else}
                <span class="flex items-center gap-1 text-xs text-text-3">
                  <Calendar size={12} />
                  {event.starts_at ? new Date(event.starts_at).toLocaleDateString(undefined, { weekday: 'short', month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit' }) : 'TBD'}
                </span>
              {/if}
            </div>
          </div>

          <h3 class="text-base font-bold leading-snug mb-1.5 font-display">{event.title}</h3>
          {#if event.description}
            <p class="text-[0.8125rem] text-text-3 leading-relaxed mb-3">{event.description}</p>
          {/if}

          <a href="/bets/new?event={event.id}" class="mt-auto self-start inline-flex items-center justify-center gap-1.5 px-3 py-1.5 font-display text-xs font-semibold bg-lime text-white rounded-md no-underline hover:bg-lime-hover active:scale-95 transition-all duration-150">
            <Zap size={12} />
            Bet on this
          </a>
        </div>
      {/each}
    </div>
  {/if}
</div>
