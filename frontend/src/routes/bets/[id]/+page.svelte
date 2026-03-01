<script lang="ts">
  import { page } from '$app/stores';

  const bet = {
    id: $page.params.id,
    question: 'Lakers defeat Celtics on Mar 5',
    creator_position: 'Lakers win',
    opponent_position: 'Celtics win',
    amount_wei: 500000000000000000,
    odds_numerator: 3,
    odds_denominator: 2,
    status: 'active',
    creator: { username: 'you', display_name: 'You' },
    opponent: { username: 'chad_bets', display_name: 'Chad' },
    created_at: '2026-03-01T10:00:00Z',
    expires_at: '2026-03-04T10:00:00Z',
    on_chain_bet_id: null,
    assertion_id: null,
  };

  function formatWei(w: number) { return (w / 1e18).toFixed(3) + ' ETH'; }
</script>

<svelte:head><title>SideBet — Bet Detail</title></svelte:head>

<div class="bet-detail" style="max-width:700px">
  <a href="/bets" class="text-sm text-muted" style="margin-bottom:var(--space-lg);display:inline-block">← Back to My Bets</a>

  <div class="card-flat animate-slide-up" style="padding:var(--space-2xl)">
    <div class="flex items-center justify-between" style="margin-bottom:var(--space-lg)">
      <span class="badge badge-blue">{bet.status}</span>
      <span class="text-xs text-muted">Created {new Date(bet.created_at).toLocaleDateString()}</span>
    </div>

    <h2 style="margin-bottom:var(--space-lg)">{bet.question}</h2>

    <div class="matchup">
      <div class="side">
        <div class="avatar">{bet.creator.display_name[0]}</div>
        <span class="side-name">@{bet.creator.username}</span>
        <span class="side-position">{bet.creator_position}</span>
      </div>
      <div class="vs-divider">VS</div>
      <div class="side">
        <div class="avatar">{bet.opponent.display_name[0]}</div>
        <span class="side-name">@{bet.opponent.username}</span>
        <span class="side-position">{bet.opponent_position}</span>
      </div>
    </div>

    <div class="detail-grid">
      <div class="detail-item"><span class="label">Wager</span><span class="val mono">{formatWei(bet.amount_wei)}</span></div>
      <div class="detail-item"><span class="label">Odds</span><span class="val">{bet.odds_numerator}:{bet.odds_denominator}</span></div>
      <div class="detail-item"><span class="label">Expires</span><span class="val">{new Date(bet.expires_at).toLocaleString()}</span></div>
      <div class="detail-item"><span class="label">On-chain ID</span><span class="val mono">{bet.on_chain_bet_id || '—'}</span></div>
    </div>

    {#if bet.status === 'proposed'}
      <div class="actions">
        <button class="btn btn-success btn-lg" style="flex:1">Accept Bet</button>
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
  .matchup { display:flex; align-items:center; justify-content:center; gap:var(--space-xl); margin-bottom:var(--space-2xl); }
  .side { display:flex; flex-direction:column; align-items:center; gap:var(--space-sm); }
  .side-name { font-weight:600; font-size:0.9375rem; }
  .side-position { font-size:0.8125rem; color:var(--text-muted); }
  .vs-divider { font-weight:800; color:var(--text-muted); font-size:0.75rem; letter-spacing:0.1em; }
  .detail-grid { display:grid; grid-template-columns:1fr 1fr; gap:var(--space-md); margin-bottom:var(--space-xl); }
  .detail-item { background:var(--bg-input); padding:var(--space-md); border-radius:var(--radius-md); }
  .detail-item .label { display:block; font-size:0.6875rem; font-weight:600; text-transform:uppercase; letter-spacing:0.05em; color:var(--text-muted); margin-bottom:4px; }
  .detail-item .val { font-weight:700; font-size:0.9375rem; }
  .mono { font-family:var(--font-mono); color:var(--accent-blue); }
  .actions { display:flex; gap:var(--space-md); }
</style>
