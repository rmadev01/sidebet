<script lang="ts">
  let activeTab = $state<'nba' | 'politics'>('nba');

  const nbaEvents = [
    { id: 'e1', home: 'Lakers', away: 'Celtics', date: 'Mar 5 · 7:30 PM', status: 'upcoming', odds: { home: 2.15, away: 1.78 } },
    { id: 'e2', home: 'Warriors', away: 'Nuggets', date: 'Mar 5 · 9:00 PM', status: 'upcoming', odds: { home: 1.95, away: 1.90 } },
    { id: 'e3', home: 'Knicks', away: '76ers', date: 'Mar 6 · 7:00 PM', status: 'upcoming', odds: { home: 1.65, away: 2.35 } },
    { id: 'e4', home: 'Bucks', away: 'Heat', date: 'Mar 6 · 8:00 PM', status: 'live', odds: { home: 1.45, away: 2.80 } },
    { id: 'e5', home: 'Suns', away: 'Mavericks', date: 'Mar 7 · 9:30 PM', status: 'upcoming', odds: { home: 2.05, away: 1.82 } },
    { id: 'e6', home: 'Clippers', away: 'Thunder', date: 'Mar 7 · 8:00 PM', status: 'upcoming', odds: { home: 2.50, away: 1.55 } },
  ];

  const politicsEvents = [
    { id: 'p1', question: 'Will the Democrats win the 2028 presidential election?', date: 'Nov 3, 2028', yes: 0.48 },
    { id: 'p2', question: 'Will there be a government shutdown in 2026?', date: 'Sep 30, 2026', yes: 0.35 },
    { id: 'p3', question: 'Will the Fed cut rates below 3% by Dec 2026?', date: 'Dec 31, 2026', yes: 0.22 },
    { id: 'p4', question: 'Will TikTok be banned in the US by 2027?', date: 'Jan 1, 2027', yes: 0.15 },
  ];

  function prob(odds: number) { return Math.round(100 / odds) + '%'; }
</script>

<svelte:head>
  <title>SideBet — Events</title>
  <meta name="description" content="Browse NBA games and political prediction markets." />
</svelte:head>

<div class="events">
  <h1 class="animate-in">Events</h1>
  <p class="subtitle animate-in" style="animation-delay:40ms">Browse games and markets to bet on</p>

  <div class="seg-control animate-in" style="animation-delay:80ms">
    <button class="seg-btn" class:active={activeTab === 'nba'} onclick={() => activeTab = 'nba'}>NBA</button>
    <button class="seg-btn" class:active={activeTab === 'politics'} onclick={() => activeTab = 'politics'}>Politics</button>
  </div>

  {#if activeTab === 'nba'}
    <div class="game-grid stagger">
      {#each nbaEvents as g}
        <a href="/bets/new?event={g.id}" class="game-card card card-interactive">
          {#if g.status === 'live'}
            <div class="live-tag"><span class="dot dot--live"></span> Live</div>
          {:else}
            <div class="game-date">{g.date}</div>
          {/if}

          <div class="matchup">
            <div class="team">
              <span class="team-name">{g.home}</span>
              <span class="team-odds mono">{g.odds.home.toFixed(2)}</span>
              <span class="team-prob">{prob(g.odds.home)}</span>
            </div>
            <span class="vs">v</span>
            <div class="team">
              <span class="team-name">{g.away}</span>
              <span class="team-odds mono">{g.odds.away.toFixed(2)}</span>
              <span class="team-prob">{prob(g.odds.away)}</span>
            </div>
          </div>
        </a>
      {/each}
    </div>
  {:else}
    <div class="markets stagger">
      {#each politicsEvents as m}
        <a href="/bets/new?event={m.id}" class="market-row">
          <div class="market-body">
            <span class="market-q">{m.question}</span>
            <span class="market-date">{m.date}</span>
          </div>
          <div class="market-bar-wrap">
            <div class="market-bar">
              <div class="bar-yes" style="width:{m.yes * 100}%">{Math.round(m.yes * 100)}%</div>
              <div class="bar-no">{Math.round((1 - m.yes) * 100)}%</div>
            </div>
            <div class="bar-labels">
              <span>Yes {(1/m.yes).toFixed(2)}</span>
              <span>No {(1/(1-m.yes)).toFixed(2)}</span>
            </div>
          </div>
        </a>
      {/each}
    </div>
  {/if}
</div>

<style>
  .events { max-width: 820px; }
  .events h1 { margin-bottom: 4px; }
  .subtitle { color: var(--text-2); font-size: 0.9375rem; margin-bottom: 24px; }

  /* ── NBA Grid ── */
  .game-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
  }

  .game-card {
    text-decoration: none;
    color: inherit;
    display: block;
    padding: 20px;
  }

  .live-tag {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-family: var(--font-display);
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--lime);
    margin-bottom: 14px;
  }

  .game-date {
    font-size: 0.75rem;
    color: var(--text-3);
    margin-bottom: 14px;
  }

  .matchup {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .team {
    flex: 1;
    text-align: center;
  }

  .team-name {
    display: block;
    font-family: var(--font-display);
    font-size: 1.125rem;
    font-weight: 700;
    margin-bottom: 6px;
  }

  .team-odds {
    display: block;
    font-size: 1.375rem;
    font-weight: 700;
    color: var(--lime);
    margin-bottom: 2px;
  }

  .team-prob {
    font-size: 0.6875rem;
    color: var(--text-3);
  }

  .vs {
    font-family: var(--font-display);
    font-size: 0.6875rem;
    font-weight: 600;
    color: var(--text-3);
    text-transform: lowercase;
  }

  /* ── Politics Markets ── */
  .markets { display: flex; flex-direction: column; gap: 2px; }

  .market-row {
    display: flex;
    align-items: center;
    gap: 24px;
    padding: 18px 0;
    border-bottom: 1px solid var(--border);
    text-decoration: none;
    color: inherit;
    transition: background var(--dur-fast);
  }
  .market-row:hover { background: var(--bg-raised); padding-left: 12px; padding-right: 12px; border-radius: var(--r-md); }
  .market-row:last-child { border-bottom: none; }

  .market-body { flex: 1; min-width: 0; }
  .market-q {
    display: block;
    font-weight: 600;
    font-size: 0.9375rem;
    line-height: 1.35;
    margin-bottom: 4px;
  }
  .market-date { font-size: 0.75rem; color: var(--text-3); }

  .market-bar-wrap { width: 200px; flex-shrink: 0; }
  .market-bar {
    display: flex;
    height: 24px;
    border-radius: var(--r-sm);
    overflow: hidden;
    font-size: 0.6875rem;
    font-weight: 700;
    font-family: var(--font-display);
  }
  .bar-yes {
    background: var(--lime);
    color: var(--bg-root);
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 32px;
  }
  .bar-no {
    flex: 1;
    background: var(--bg-hover);
    color: var(--text-2);
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 32px;
  }
  .bar-labels {
    display: flex;
    justify-content: space-between;
    font-size: 0.6875rem;
    color: var(--text-3);
    margin-top: 4px;
    font-family: var(--font-mono);
  }

  @media (max-width: 640px) {
    .market-row { flex-direction: column; align-items: flex-start; gap: 10px; }
    .market-bar-wrap { width: 100%; }
  }
</style>
