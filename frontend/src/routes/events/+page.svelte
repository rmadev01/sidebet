<script lang="ts">
  import { onMount } from 'svelte';

  let activeTab = $state<'nba' | 'politics'>('nba');

  const demoNbaEvents = [
    { id: 'e1', title: 'Lakers vs Celtics', starts_at: '2026-03-05T19:30:00Z', status: 'upcoming', category: 'nba', cached_odds: { home: 2.15, away: 1.78 }, sport: 'Basketball', league: 'NBA' },
    { id: 'e2', title: 'Warriors vs Nuggets', starts_at: '2026-03-05T21:00:00Z', status: 'upcoming', category: 'nba', cached_odds: { home: 1.95, away: 1.90 }, sport: 'Basketball', league: 'NBA' },
    { id: 'e3', title: 'Knicks vs 76ers', starts_at: '2026-03-06T19:00:00Z', status: 'upcoming', category: 'nba', cached_odds: { home: 1.65, away: 2.35 }, sport: 'Basketball', league: 'NBA' },
    { id: 'e4', title: 'Bucks vs Heat', starts_at: '2026-03-06T20:00:00Z', status: 'live', category: 'nba', cached_odds: { home: 1.45, away: 2.80 }, sport: 'Basketball', league: 'NBA' },
    { id: 'e5', title: 'Suns vs Mavericks', starts_at: '2026-03-07T21:30:00Z', status: 'upcoming', category: 'nba', cached_odds: { home: 2.05, away: 1.82 }, sport: 'Basketball', league: 'NBA' },
    { id: 'e6', title: 'Clippers vs Thunder', starts_at: '2026-03-07T20:00:00Z', status: 'upcoming', category: 'nba', cached_odds: { home: 2.50, away: 1.55 }, sport: 'Basketball', league: 'NBA' },
  ];

  const demoPoliticsEvents = [
    { id: 'p1', title: 'Will the Democrats win the 2028 presidential election?', starts_at: '2028-11-03T00:00:00Z', status: 'upcoming', category: 'politics', cached_odds: { yes: 0.48, no: 0.52 } },
    { id: 'p2', title: 'Will there be a government shutdown in 2026?', starts_at: '2026-09-30T00:00:00Z', status: 'upcoming', category: 'politics', cached_odds: { yes: 0.35, no: 0.65 } },
    { id: 'p3', title: 'Will the Fed cut rates below 3% by Dec 2026?', starts_at: '2026-12-31T00:00:00Z', status: 'upcoming', category: 'politics', cached_odds: { yes: 0.22, no: 0.78 } },
    { id: 'p4', title: 'Will TikTok be banned in the US by 2027?', starts_at: '2027-01-01T00:00:00Z', status: 'upcoming', category: 'politics', cached_odds: { yes: 0.15, no: 0.85 } },
  ];

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit' });
  }

  function oddsToProb(odds: number): string {
    return (100 / odds).toFixed(0) + '%';
  }
</script>

<svelte:head>
  <title>SideBet — Events</title>
  <meta name="description" content="Browse NBA games and political prediction markets to bet on." />
</svelte:head>

<div class="events-page">
  <div class="page-header animate-slide-up">
    <h1>Events</h1>
    <p class="text-secondary">Browse games and markets to bet on</p>
  </div>

  <div class="tabs animate-slide-up" style="animation-delay: 60ms">
    <button class="tab" class:active={activeTab === 'nba'} onclick={() => activeTab = 'nba'}>
      🏀 NBA
    </button>
    <button class="tab" class:active={activeTab === 'politics'} onclick={() => activeTab = 'politics'}>
      🏛️ Politics
    </button>
  </div>

  {#if activeTab === 'nba'}
    <div class="grid-cards stagger">
      {#each demoNbaEvents as event}
        <div class="card event-card">
          <div class="event-card-header">
            <span class="badge" class:badge-green={event.status === 'live'} class:badge-blue={event.status === 'upcoming'}>
              {#if event.status === 'live'}<span class="status-dot live"></span>{/if}
              {event.status}
            </span>
            <span class="text-xs text-muted">{formatDate(event.starts_at)}</span>
          </div>

          <h3 class="event-title">{event.title}</h3>

          <div class="odds-row">
            <div class="odds-box">
              <span class="odds-label">{event.title.split(' vs ')[0]}</span>
              <span class="odds-value">{event.cached_odds.home.toFixed(2)}</span>
              <span class="odds-prob">{oddsToProb(event.cached_odds.home)}</span>
            </div>
            <div class="odds-vs">VS</div>
            <div class="odds-box">
              <span class="odds-label">{event.title.split(' vs ')[1]}</span>
              <span class="odds-value">{event.cached_odds.away.toFixed(2)}</span>
              <span class="odds-prob">{oddsToProb(event.cached_odds.away)}</span>
            </div>
          </div>

          <a href="/bets/new?event={event.id}" class="btn btn-primary btn-sm" style="width:100%; margin-top: var(--space-md)">
            Bet on this game
          </a>
        </div>
      {/each}
    </div>
  {:else}
    <div class="grid-cards stagger">
      {#each demoPoliticsEvents as event}
        <div class="card event-card">
          <div class="event-card-header">
            <span class="badge badge-blue">Market</span>
            <span class="text-xs text-muted">{formatDate(event.starts_at)}</span>
          </div>

          <h3 class="event-title">{event.title}</h3>

          <div class="market-odds">
            <div class="market-bar">
              <div class="market-yes" style="width: {event.cached_odds.yes * 100}%">
                Yes {(event.cached_odds.yes * 100).toFixed(0)}%
              </div>
              <div class="market-no">
                No {(event.cached_odds.no * 100).toFixed(0)}%
              </div>
            </div>
            <div class="market-decimal">
              <span>Yes: {(1/event.cached_odds.yes).toFixed(2)}</span>
              <span>No: {(1/event.cached_odds.no).toFixed(2)}</span>
            </div>
          </div>

          <a href="/bets/new?event={event.id}" class="btn btn-primary btn-sm" style="width:100%; margin-top: var(--space-md)">
            Bet on this market
          </a>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .events-page { max-width: 1100px; }
  .page-header {
    margin-bottom: var(--space-lg);
  }
  .page-header h1 { margin-bottom: 4px; }

  .event-card { cursor: default; }
  .event-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-md);
  }

  .event-title {
    font-size: 1.0625rem;
    margin-bottom: var(--space-md);
  }

  /* NBA odds */
  .odds-row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }
  .odds-box {
    flex: 1;
    background: var(--bg-input);
    border-radius: var(--radius-md);
    padding: var(--space-md);
    text-align: center;
  }
  .odds-label {
    display: block;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 4px;
  }
  .odds-value {
    display: block;
    font-size: 1.5rem;
    font-weight: 800;
    color: var(--accent-blue);
    font-family: var(--font-mono);
    letter-spacing: -0.02em;
  }
  .odds-prob {
    display: block;
    font-size: 0.75rem;
    color: var(--text-secondary);
    margin-top: 2px;
  }
  .odds-vs {
    font-size: 0.6875rem;
    font-weight: 800;
    color: var(--text-muted);
    letter-spacing: 0.1em;
  }

  /* Politics market bar */
  .market-odds { margin-bottom: var(--space-sm); }
  .market-bar {
    display: flex;
    height: 32px;
    border-radius: var(--radius-sm);
    overflow: hidden;
    font-size: 0.75rem;
    font-weight: 700;
  }
  .market-yes {
    background: var(--accent-green);
    color: var(--text-inverse);
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 40px;
    transition: width var(--transition-base);
  }
  .market-no {
    flex: 1;
    background: var(--accent-red);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 40px;
  }
  .market-decimal {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-top: var(--space-xs);
  }
</style>
