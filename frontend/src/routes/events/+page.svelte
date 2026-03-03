<script lang="ts">
  import { onMount } from 'svelte';
  import { getEvents } from '$lib/api';

  let events = $state<any[]>([]);
  let category = $state('all');
  let loading = $state(true);

  onMount(async () => {
    try {
      events = await getEvents();
    } catch { /* fallback to empty */ }
    loading = false;
  });

  function filtered() {
    if (category === 'all') return events;
    return events.filter(e => e.category === category);
  }

  function categories() {
    const cats = [...new Set(events.map(e => e.category))];
    return ['all', ...cats];
  }
</script>

<svelte:head>
  <title>Events — SideBet</title>
  <meta name="description" content="Browse events to bet on." />
</svelte:head>

<div class="events-page">
  <div class="page-head">
    <h1>Events</h1>
  </div>

  <!-- Category tabs -->
  <div class="tabs">
    {#each categories() as cat}
      <button class="tab" class:active={category === cat} onclick={() => category = cat}>
        {cat === 'all' ? 'All' : cat.charAt(0).toUpperCase() + cat.slice(1)}
      </button>
    {/each}
  </div>

  {#if loading}
    <p class="empty-state">Loading…</p>
  {:else if filtered().length === 0}
    <p class="empty-state">No events available right now.</p>
  {:else}
    <div class="event-grid stagger">
      {#each filtered() as event}
        <div class="event-card animate-in">
          <div class="ev-head">
            <span class="ev-cat">{event.category}</span>
            {#if event.status === 'live'}
              <span class="ev-live"><span class="dot dot--live"></span>LIVE</span>
            {:else}
              <span class="ev-time">{event.starts_at ? new Date(event.starts_at).toLocaleDateString() : 'TBD'}</span>
            {/if}
          </div>
          <h3 class="ev-title">{event.title}</h3>
          {#if event.description}
            <p class="ev-desc">{event.description}</p>
          {/if}
          {#if event.cached_odds && Object.keys(event.cached_odds).length > 0}
            <div class="ev-odds">
              {#each Object.entries(event.cached_odds) as [key, val]}
                <span class="odd-tag">{key}: <span class="mono">{typeof val === 'number' ? val.toFixed(2) : val}</span></span>
              {/each}
            </div>
          {/if}
          <a href="/bets/new?event={event.id}" class="btn btn-primary btn-sm ev-btn">Bet on this</a>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .events-page { max-width: 860px; }

  .page-head {
    margin-bottom: 24px;
  }
  .page-head h1 { font-size: 1.5rem; }

  .tabs {
    display: flex;
    gap: 6px;
    margin-bottom: 24px;
    flex-wrap: wrap;
  }
  .tab {
    padding: 6px 16px;
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--text-3);
    background: var(--bg-raised);
    border: 1px solid var(--border);
    border-radius: var(--r-full);
    cursor: pointer;
    transition: all var(--dur-fast) var(--ease-out);
  }
  .tab:hover { color: var(--text-2); border-color: var(--text-3); }
  .tab.active { background: var(--lime-dim); border-color: var(--lime); color: var(--lime); }

  .empty-state {
    color: var(--text-3);
    font-size: 0.875rem;
    padding: 40px 0;
    text-align: center;
  }

  .event-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
  }

  .event-card {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    padding: 20px;
    display: flex;
    flex-direction: column;
    transition: border-color var(--dur-fast) var(--ease-out);
  }
  .event-card:hover { border-color: var(--text-3); }

  .ev-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
  }
  .ev-cat {
    font-size: 0.6875rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--sky);
  }
  .ev-live {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 0.6875rem;
    font-weight: 700;
    color: var(--rose);
  }
  .ev-time {
    font-size: 0.75rem;
    color: var(--text-3);
  }

  .ev-title {
    font-size: 1rem;
    font-weight: 700;
    margin-bottom: 6px;
    line-height: 1.3;
  }

  .ev-desc {
    font-size: 0.8125rem;
    color: var(--text-3);
    line-height: 1.4;
    margin-bottom: 12px;
  }

  .ev-odds {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 14px;
  }
  .odd-tag {
    font-size: 0.75rem;
    background: var(--bg-raised);
    padding: 3px 8px;
    border-radius: var(--r-sm);
    color: var(--text-2);
  }

  .ev-btn {
    margin-top: auto;
    align-self: flex-start;
  }
</style>
