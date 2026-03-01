<script lang="ts">
  import { user } from '$lib/stores';

  const stats = {
    wins: 12,
    losses: 8,
    winRate: 60.0,
    streak: 3,
    streakType: 'W',
    totalWagered: '2.450',
    totalBets: 20,
    activeBets: 3,
    avgBet: '0.123',
    bestWin: '1.000',
    accuracy: { nba: 66.7, politics: 50.0 },
  };

  // SVG donut for win rate
  const circumference = 2 * Math.PI * 44;
  const winOffset = circumference - (stats.winRate / 100) * circumference;
</script>

<svelte:head><title>SideBet — Profile</title></svelte:head>

<div class="profile">
  <!-- Header -->
  <div class="prof-head animate-in">
    <div class="av av--lg av-1">{$user?.name?.[0]?.toUpperCase() || 'U'}</div>
    <div class="prof-info">
      <h1>{$user?.name || 'Username'}</h1>
      <span class="handle">@{$user?.name?.toLowerCase()?.replace(/\s/g, '_') || 'user'}</span>
    </div>
  </div>

  <!-- Win Rate Ring + Record -->
  <div class="record-section animate-in" style="animation-delay:60ms">
    <div class="ring-wrap">
      <svg viewBox="0 0 100 100" class="ring-svg">
        <circle cx="50" cy="50" r="44" fill="none" stroke="var(--bg-raised)" stroke-width="6" />
        <circle cx="50" cy="50" r="44" fill="none" stroke="var(--lime)" stroke-width="6"
          stroke-linecap="round"
          stroke-dasharray={circumference}
          stroke-dashoffset={winOffset}
          transform="rotate(-90 50 50)"
          class="ring-fill" />
      </svg>
      <div class="ring-center">
        <span class="ring-pct">{stats.winRate}%</span>
        <span class="ring-label">win rate</span>
      </div>
    </div>

    <div class="record-nums">
      <div class="rec">
        <span class="rec-n" style="color:var(--lime)">{stats.wins}</span>
        <span class="rec-l">Wins</span>
      </div>
      <div class="rec-sep"></div>
      <div class="rec">
        <span class="rec-n" style="color:var(--rose)">{stats.losses}</span>
        <span class="rec-l">Losses</span>
      </div>
      <div class="rec-sep"></div>
      <div class="rec">
        <span class="rec-n" style="color:var(--amber)">{stats.streak}{stats.streakType}</span>
        <span class="rec-l">Streak</span>
      </div>
    </div>
  </div>

  <!-- Stats Strip -->
  <div class="stats-strip animate-in" style="animation-delay:120ms">
    <div class="stat-item"><span class="stat-val mono">{stats.totalWagered}</span><span class="stat-unit">ETH wagered</span></div>
    <div class="stat-item"><span class="stat-val mono">{stats.totalBets}</span><span class="stat-unit">total bets</span></div>
    <div class="stat-item"><span class="stat-val mono">{stats.activeBets}</span><span class="stat-unit">active</span></div>
    <div class="stat-item"><span class="stat-val mono">{stats.avgBet}</span><span class="stat-unit">avg bet</span></div>
    <div class="stat-item"><span class="stat-val mono">{stats.bestWin}</span><span class="stat-unit">best win</span></div>
  </div>

  <!-- Accuracy -->
  <section class="accuracy-section animate-in" style="animation-delay:180ms">
    <h3>Accuracy by Category</h3>
    <div class="acc-bars">
      <div class="acc-row">
        <span class="acc-label">NBA</span>
        <div class="acc-track">
          <div class="acc-fill" style="width:{stats.accuracy.nba}%; background:var(--lime)">
            <span class="acc-pct">{stats.accuracy.nba}%</span>
          </div>
        </div>
      </div>
      <div class="acc-row">
        <span class="acc-label">Politics</span>
        <div class="acc-track">
          <div class="acc-fill" style="width:{stats.accuracy.politics}%; background:var(--sky)">
            <span class="acc-pct">{stats.accuracy.politics}%</span>
          </div>
        </div>
      </div>
    </div>
  </section>
</div>

<style>
  .profile { max-width: 640px; }

  /* ── Header ── */
  .prof-head {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 36px;
  }
  .prof-info h1 { margin-bottom: 2px; }
  .handle { font-size: 0.875rem; color: var(--text-3); }

  /* ── Record Section ── */
  .record-section {
    display: flex;
    align-items: center;
    gap: 40px;
    margin-bottom: 36px;
    padding: 28px 32px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-xl);
  }

  .ring-wrap {
    position: relative;
    width: 100px;
    height: 100px;
    flex-shrink: 0;
  }
  .ring-svg { width: 100%; height: 100%; }
  .ring-fill { transition: stroke-dashoffset 0.8s var(--ease-out); }
  .ring-center {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
  }
  .ring-pct {
    font-family: var(--font-display);
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--lime);
    line-height: 1;
  }
  .ring-label { font-size: 0.625rem; color: var(--text-3); text-transform: uppercase; letter-spacing: 0.06em; margin-top: 2px; }

  .record-nums {
    display: flex;
    align-items: center;
    gap: 24px;
  }
  .rec { text-align: center; }
  .rec-n {
    display: block;
    font-family: var(--font-display);
    font-size: 2rem;
    font-weight: 700;
    line-height: 1;
    margin-bottom: 4px;
  }
  .rec-l {
    font-size: 0.6875rem;
    color: var(--text-3);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 600;
  }
  .rec-sep {
    width: 1px;
    height: 32px;
    background: var(--border);
  }

  /* ── Stats Strip ── */
  .stats-strip {
    display: flex;
    gap: 6px;
    overflow-x: auto;
    padding: 4px 0;
    margin-bottom: 36px;
    -webkit-overflow-scrolling: touch;
  }
  .stat-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 14px 18px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    flex-shrink: 0;
    min-width: 100px;
  }
  .stat-val {
    font-size: 1.125rem;
    font-weight: 700;
    color: var(--text-1);
    margin-bottom: 2px;
  }
  .stat-unit {
    font-size: 0.625rem;
    color: var(--text-3);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: 600;
  }

  /* ── Accuracy ── */
  .accuracy-section h3 { margin-bottom: 16px; }
  .acc-bars { display: flex; flex-direction: column; gap: 12px; }
  .acc-row { display: flex; align-items: center; gap: 14px; }
  .acc-label {
    width: 70px;
    font-family: var(--font-display);
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-2);
    flex-shrink: 0;
  }
  .acc-track {
    flex: 1;
    height: 24px;
    background: var(--bg-raised);
    border-radius: var(--r-md);
    overflow: hidden;
  }
  .acc-fill {
    height: 100%;
    border-radius: var(--r-md);
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding-right: 8px;
    transition: width 0.6s var(--ease-out);
  }
  .acc-pct {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    font-weight: 700;
    color: var(--bg-root);
  }

  @media (max-width: 600px) {
    .record-section { flex-direction: column; gap: 20px; padding: 20px; }
    .record-nums { gap: 16px; }
    .prof-head { flex-direction: column; text-align: center; }
  }
</style>
