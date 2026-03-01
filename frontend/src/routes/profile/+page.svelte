<script lang="ts">
  import { user, isAuthenticated } from '$lib/stores';

  const stats = {
    wins: 12,
    losses: 8,
    winRate: '60.0',
    streak: 3,
    streakType: 'W',
    totalWagered: '2.450',
    totalBets: 20,
    activeBets: 3,
    avgBetSize: '0.123',
    bestWin: '1.000',
    accuracy: {
      nba: '66.7',
      politics: '50.0',
    }
  };
</script>

<svelte:head>
  <title>SideBet — Profile</title>
</svelte:head>

<div class="profile-page">
  <div class="profile-header animate-slide-up">
    <div class="avatar avatar-lg">
      {$user?.name?.[0]?.toUpperCase() || 'U'}
    </div>
    <div class="profile-info">
      <h1>{$user?.name || 'Username'}</h1>
      <p class="text-secondary">@{$user?.name?.toLowerCase()?.replace(/\s/g, '_') || 'user'}</p>
    </div>
    <button class="btn btn-ghost">Edit Profile</button>
  </div>

  <!-- Win/Loss Banner -->
  <div class="wl-banner animate-slide-up" style="animation-delay: 60ms">
    <div class="wl-main">
      <span class="wl-wins">{stats.wins}</span>
      <span class="wl-separator">—</span>
      <span class="wl-losses">{stats.losses}</span>
    </div>
    <span class="wl-rate">{stats.winRate}% Win Rate</span>
    <div class="streak-badge">
      🔥 {stats.streak}{stats.streakType} Streak
    </div>
  </div>

  <!-- Stats Grid -->
  <div class="stats-grid stagger">
    <div class="card stat-tile">
      <span class="stat-tile-value">{stats.totalWagered} ETH</span>
      <span class="stat-tile-label">Total Wagered</span>
    </div>
    <div class="card stat-tile">
      <span class="stat-tile-value">{stats.totalBets}</span>
      <span class="stat-tile-label">Total Bets</span>
    </div>
    <div class="card stat-tile">
      <span class="stat-tile-value">{stats.activeBets}</span>
      <span class="stat-tile-label">Active</span>
    </div>
    <div class="card stat-tile">
      <span class="stat-tile-value">{stats.avgBetSize} ETH</span>
      <span class="stat-tile-label">Avg Bet Size</span>
    </div>
    <div class="card stat-tile">
      <span class="stat-tile-value">{stats.bestWin} ETH</span>
      <span class="stat-tile-label">Best Win</span>
    </div>
    <div class="card stat-tile">
      <span class="stat-tile-value">—</span>
      <span class="stat-tile-label">Wallet</span>
    </div>
  </div>

  <!-- Accuracy by Category -->
  <div class="accuracy-section animate-slide-up" style="animation-delay: 240ms">
    <h3>Accuracy by Category</h3>
    <div class="accuracy-bars">
      <div class="accuracy-row">
        <span class="accuracy-label">🏀 NBA</span>
        <div class="accuracy-bar-track">
          <div class="accuracy-bar-fill" style="width: {stats.accuracy.nba}%; background: var(--accent-green)"></div>
        </div>
        <span class="accuracy-value">{stats.accuracy.nba}%</span>
      </div>
      <div class="accuracy-row">
        <span class="accuracy-label">🏛️ Politics</span>
        <div class="accuracy-bar-track">
          <div class="accuracy-bar-fill" style="width: {stats.accuracy.politics}%; background: var(--accent-blue)"></div>
        </div>
        <span class="accuracy-value">{stats.accuracy.politics}%</span>
      </div>
    </div>
  </div>
</div>

<style>
  .profile-page { max-width: 800px; }

  .profile-header {
    display: flex;
    align-items: center;
    gap: var(--space-lg);
    margin-bottom: var(--space-2xl);
  }
  .profile-info h1 { margin-bottom: 2px; }

  /* W/L Banner */
  .wl-banner {
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
    padding: var(--space-xl) var(--space-2xl);
    text-align: center;
    margin-bottom: var(--space-2xl);
  }
  .wl-main {
    font-size: 3rem;
    font-weight: 800;
    letter-spacing: -0.04em;
    line-height: 1;
    margin-bottom: var(--space-sm);
  }
  .wl-wins { color: var(--accent-green); }
  .wl-separator { color: var(--text-muted); margin: 0 var(--space-md); }
  .wl-losses { color: var(--accent-red); }
  .wl-rate {
    display: block;
    font-size: 0.9375rem;
    color: var(--text-secondary);
    margin-bottom: var(--space-sm);
  }
  .streak-badge {
    display: inline-block;
    background: var(--accent-gold-dim);
    color: var(--accent-gold);
    padding: var(--space-xs) var(--space-md);
    border-radius: var(--radius-full);
    font-size: 0.8125rem;
    font-weight: 700;
  }

  /* Stats Grid */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-md);
    margin-bottom: var(--space-2xl);
  }
  .stat-tile {
    text-align: center;
    padding: var(--space-lg);
  }
  .stat-tile-value {
    display: block;
    font-size: 1.25rem;
    font-weight: 800;
    color: var(--accent-blue);
    font-family: var(--font-mono);
    margin-bottom: 4px;
  }
  .stat-tile-label {
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
  }

  /* Accuracy */
  .accuracy-section h3 { margin-bottom: var(--space-lg); }
  .accuracy-bars { display: flex; flex-direction: column; gap: var(--space-md); }
  .accuracy-row {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }
  .accuracy-label {
    width: 100px;
    font-size: 0.875rem;
    font-weight: 500;
  }
  .accuracy-bar-track {
    flex: 1;
    height: 8px;
    background: var(--bg-input);
    border-radius: var(--radius-full);
    overflow: hidden;
  }
  .accuracy-bar-fill {
    height: 100%;
    border-radius: var(--radius-full);
    transition: width var(--transition-slow);
  }
  .accuracy-value {
    width: 50px;
    text-align: right;
    font-size: 0.875rem;
    font-weight: 700;
    font-family: var(--font-mono);
  }

  @media (max-width: 640px) {
    .stats-grid { grid-template-columns: repeat(2, 1fr); }
    .profile-header { flex-direction: column; text-align: center; }
  }
</style>
