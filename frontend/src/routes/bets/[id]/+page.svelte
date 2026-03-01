<script lang="ts">
  import { page } from '$app/stores';

  const bet = {
    id: $page.params.id,
    question: 'Lakers defeat Celtics on Mar 5',
    creator_position: 'Lakers win',
    opponent_position: 'Celtics win',
    amount: 0.5,
    odds: '3:2',
    status: 'active',
    creator: { username: 'you', name: 'You', av: 1 },
    opponent: { username: 'chad_bets', name: 'Chad', av: 2 },
    created_at: '2026-03-01',
    expires_at: '2026-03-04',
    on_chain_id: null as string | null,
    timeline: [
      { label: 'Created', date: 'Mar 1, 2:15 PM', done: true },
      { label: 'Accepted', date: 'Mar 1, 3:42 PM', done: true },
      { label: 'Settling', date: '—', done: false },
      { label: 'Settled', date: '—', done: false },
    ],
  };
</script>

<svelte:head><title>SideBet — Bet Detail</title></svelte:head>

<div class="detail">
  <a href="/bets" class="back-link animate-in">← Back to bets</a>

  <div class="detail-card animate-in" style="animation-delay:60ms">
    <h2>{bet.question}</h2>

    <!-- Matchup -->
    <div class="matchup">
      <div class="side">
        <div class="av av-{bet.creator.av}">{bet.creator.name[0]}</div>
        <span class="side-name">@{bet.creator.username}</span>
        <span class="side-pos">{bet.creator_position}</span>
      </div>
      <span class="vs-mark">VS</span>
      <div class="side">
        <div class="av av-{bet.opponent.av}">{bet.opponent.name[0]}</div>
        <span class="side-name">@{bet.opponent.username}</span>
        <span class="side-pos">{bet.opponent_position}</span>
      </div>
    </div>

    <!-- Details Row -->
    <div class="detail-row">
      <div class="d-item"><span class="d-label">Wager</span><span class="d-val mono">{bet.amount} ETH</span></div>
      <div class="d-item"><span class="d-label">Odds</span><span class="d-val">{bet.odds}</span></div>
      <div class="d-item"><span class="d-label">Expires</span><span class="d-val">{bet.expires_at}</span></div>
      <div class="d-item"><span class="d-label">Chain ID</span><span class="d-val mono">{bet.on_chain_id || '—'}</span></div>
    </div>

    <!-- Timeline -->
    <div class="timeline">
      {#each bet.timeline as step, i}
        <div class="tl-step" class:done={step.done} class:last={i === bet.timeline.length - 1}>
          <div class="tl-dot"></div>
          <div class="tl-content">
            <span class="tl-label">{step.label}</span>
            <span class="tl-date">{step.date}</span>
          </div>
        </div>
      {/each}
    </div>

    <!-- Actions -->
    {#if bet.status === 'proposed'}
      <div class="actions">
        <button class="btn btn-accept btn-lg" style="flex:1">Accept Bet</button>
        <button class="btn btn-danger btn-lg">Decline</button>
      </div>
    {:else if bet.status === 'active'}
      <div class="actions">
        <button class="btn btn-primary btn-lg" style="flex:1">Trigger Settlement</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .detail { max-width: 620px; }

  .back-link {
    display: inline-block;
    font-size: 0.8125rem;
    color: var(--text-3);
    margin-bottom: 20px;
    text-decoration: none;
  }
  .back-link:hover { color: var(--text-2); }

  .detail-card {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-xl);
    padding: 32px;
  }

  .detail-card h2 { margin-bottom: 28px; }

  /* ── Matchup ── */
  .matchup {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 32px;
    margin-bottom: 28px;
  }
  .side {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    text-align: center;
  }
  .side-name { font-weight: 600; font-size: 0.875rem; }
  .side-pos { font-size: 0.75rem; color: var(--text-3); }
  .vs-mark {
    font-family: var(--font-display);
    font-size: 0.6875rem;
    font-weight: 700;
    color: var(--text-3);
    letter-spacing: 0.1em;
  }

  /* ── Detail Row ── */
  .detail-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 28px;
  }
  .d-item {
    padding: 12px;
    background: var(--bg-raised);
    border-radius: var(--r-md);
  }
  .d-label {
    display: block;
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-3);
    margin-bottom: 4px;
    font-family: var(--font-display);
  }
  .d-val { font-weight: 700; font-size: 0.875rem; }

  /* ── Timeline ── */
  .timeline {
    margin-bottom: 24px;
    padding-left: 8px;
  }
  .tl-step {
    display: flex;
    align-items: flex-start;
    gap: 14px;
    padding-bottom: 16px;
    position: relative;
  }
  .tl-step:not(.last)::after {
    content: '';
    position: absolute;
    left: 4px;
    top: 14px;
    bottom: 0;
    width: 1px;
    background: var(--border);
  }
  .tl-step.done:not(.last)::after { background: var(--lime); }
  .tl-dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--border);
    flex-shrink: 0;
    margin-top: 4px;
  }
  .tl-step.done .tl-dot { background: var(--lime); box-shadow: 0 0 6px var(--lime); }
  .tl-content { flex: 1; }
  .tl-label { font-size: 0.8125rem; font-weight: 600; display: block; }
  .tl-date { font-size: 0.6875rem; color: var(--text-3); }

  /* ── Actions ── */
  .actions { display: flex; gap: 10px; }

  @media (max-width: 600px) {
    .detail-card { padding: 20px; }
    .matchup { gap: 16px; }
    .detail-row { grid-template-columns: 1fr; }
  }
</style>
