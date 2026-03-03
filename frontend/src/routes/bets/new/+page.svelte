<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { isAuthenticated, coinBalance } from '$lib/stores';
  import { getFriends, getEvents, createBet } from '$lib/api';

  let step = $state(1);
  let friends = $state<any[]>([]);
  let events = $state<any[]>([]);
  let loading = $state(true);
  let submitting = $state(false);
  let error = $state('');

  let form = $state({
    opponent_id: '',
    event_id: '',
    question: '',
    creator_position: '',
    opponent_position: '',
    amount: 50,
    odds_numerator: 1,
    odds_denominator: 1,
    reference_odds: null as any,
    expires_in_hours: 24,
  });

  let selectedFriend = $state<any>(null);
  let selectedEvent = $state<any>(null);

  onMount(async () => {
    if (!$isAuthenticated) { loading = false; return; }
    try {
      const [f, e] = await Promise.all([
        getFriends().catch(() => []),
        getEvents().catch(() => []),
      ]);
      friends = f;
      events = e;
    } catch { /* ignore */ }
    loading = false;
  });

  function selectFriend(f: any) {
    selectedFriend = f;
    form.opponent_id = f.id;
  }

  function selectEvent(e: any) {
    selectedEvent = e;
    form.event_id = e.id;
    form.question = e.title;
    if (e.cached_odds && typeof e.cached_odds === 'object') {
      form.reference_odds = e.cached_odds;
    }
  }

  async function submit() {
    if (submitting) return;
    error = '';

    if (!form.opponent_id) { error = 'Select an opponent'; return; }
    if (!form.event_id) { error = 'Select an event'; return; }
    if (!form.creator_position) { error = 'Define your position'; return; }
    if (form.amount <= 0) { error = 'Amount must be positive'; return; }
    if (form.amount > $coinBalance) { error = 'Insufficient coins'; return; }

    submitting = true;
    try {
      const bet = await createBet({
        ...form,
        opponent_position: form.opponent_position || `Not: ${form.creator_position}`,
      });
      coinBalance.update(b => b - form.amount);
      goto(`/bets/${bet.id}`);
    } catch (e: any) {
      error = e.message || 'Failed to create bet';
      submitting = false;
    }
  }
</script>

<svelte:head>
  <title>New Bet — SideBet</title>
</svelte:head>

<div class="new-bet">
  <h1>Create a Bet</h1>

  <!-- Progress -->
  <div class="progress">
    {#each [1,2,3,4] as s}
      <div class="prog-dot" class:active={step >= s} class:current={step === s}></div>
      {#if s < 4}<div class="prog-line" class:active={step > s}></div>{/if}
    {/each}
  </div>

  <!-- Step 1: Select Opponent -->
  {#if step === 1}
    <div class="step-card animate-in">
      <h2>Who are you betting?</h2>
      {#if friends.length === 0}
        <p class="note">No friends yet. <a href="/friends">Add some first!</a></p>
      {:else}
        <div class="friend-grid">
          {#each friends as f}
            <button class="friend-option" class:selected={form.opponent_id === f.id} onclick={() => selectFriend(f)}>
              <span class="av av-1">{f.display_name?.[0]?.toUpperCase() || '?'}</span>
              <span class="f-name">{f.display_name || f.username}</span>
              <span class="f-handle">@{f.username}</span>
            </button>
          {/each}
        </div>
        <button class="btn btn-primary" disabled={!form.opponent_id} onclick={() => step = 2}>Continue →</button>
      {/if}
    </div>

  <!-- Step 2: Select Event -->
  {:else if step === 2}
    <div class="step-card animate-in">
      <h2>What event?</h2>
      {#if events.length === 0}
        <p class="note">No events available right now. Check back later!</p>
      {:else}
        <div class="event-list">
          {#each events as e}
            <button class="event-option" class:selected={form.event_id === e.id} onclick={() => selectEvent(e)}>
              <span class="ev-cat">{e.category}</span>
              <span class="ev-title">{e.title}</span>
              {#if e.starts_at}
                <span class="ev-time">{new Date(e.starts_at).toLocaleDateString()}</span>
              {/if}
            </button>
          {/each}
        </div>
        <div class="step-actions">
          <button class="btn btn-ghost" onclick={() => step = 1}>← Back</button>
          <button class="btn btn-primary" disabled={!form.event_id} onclick={() => step = 3}>Continue →</button>
        </div>
      {/if}
    </div>

  <!-- Step 3: Bet Details -->
  {:else if step === 3}
    <div class="step-card animate-in">
      <h2>Bet details</h2>
      <div class="form-group">
        <label>Your position</label>
        <input bind:value={form.creator_position} placeholder="e.g. Lakers win" class="input" />
      </div>
      <div class="form-group">
        <label>Opponent's position</label>
        <input bind:value={form.opponent_position} placeholder="e.g. Celtics win" class="input" />
      </div>
      <div class="form-row">
        <div class="form-group">
          <label>Amount (coins)</label>
          <input type="number" bind:value={form.amount} min="1" max={$coinBalance} class="input" />
          <span class="hint">Balance: {$coinBalance.toLocaleString()} coins</span>
        </div>
        <div class="form-group">
          <label>Odds</label>
          <div class="odds-input">
            <input type="number" bind:value={form.odds_numerator} min="1" max="100" class="input" />
            <span>:</span>
            <input type="number" bind:value={form.odds_denominator} min="1" max="100" class="input" />
          </div>
        </div>
      </div>
      <div class="step-actions">
        <button class="btn btn-ghost" onclick={() => step = 2}>← Back</button>
        <button class="btn btn-primary" disabled={!form.creator_position} onclick={() => step = 4}>Review →</button>
      </div>
    </div>

  <!-- Step 4: Review & Submit -->
  {:else if step === 4}
    <div class="step-card animate-in">
      <h2>Review your bet</h2>
      <div class="review-card">
        <div class="review-row">
          <span class="review-label">Opponent</span>
          <span class="review-val">{selectedFriend?.display_name || selectedFriend?.username}</span>
        </div>
        <div class="review-row">
          <span class="review-label">Event</span>
          <span class="review-val">{selectedEvent?.title}</span>
        </div>
        <div class="review-row">
          <span class="review-label">Your pick</span>
          <span class="review-val">{form.creator_position}</span>
        </div>
        <div class="review-row">
          <span class="review-label">Their pick</span>
          <span class="review-val">{form.opponent_position || `Not: ${form.creator_position}`}</span>
        </div>
        <div class="review-row">
          <span class="review-label">Amount</span>
          <span class="review-val mono">{form.amount.toLocaleString()} coins</span>
        </div>
        <div class="review-row">
          <span class="review-label">Odds</span>
          <span class="review-val mono">{form.odds_numerator}:{form.odds_denominator}</span>
        </div>
      </div>

      {#if error}
        <p class="error">{error}</p>
      {/if}

      <div class="step-actions">
        <button class="btn btn-ghost" onclick={() => step = 3}>← Back</button>
        <button class="btn btn-primary" onclick={submit} disabled={submitting}>
          {submitting ? 'Sending…' : 'Send Bet'}
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .new-bet { max-width: 580px; }

  h1 { font-size: 1.5rem; margin-bottom: 24px; }

  /* Progress */
  .progress {
    display: flex;
    align-items: center;
    gap: 0;
    margin-bottom: 32px;
  }
  .prog-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--bg-raised);
    border: 2px solid var(--border);
    transition: all var(--dur-base) var(--ease-out);
  }
  .prog-dot.active {
    background: var(--lime);
    border-color: var(--lime);
  }
  .prog-dot.current {
    box-shadow: 0 0 0 3px var(--lime-dim);
  }
  .prog-line {
    flex: 1;
    height: 2px;
    background: var(--border);
    transition: background var(--dur-base) var(--ease-out);
  }
  .prog-line.active { background: var(--lime); }

  .step-card {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    padding: 28px;
  }
  .step-card h2 {
    font-size: 1.125rem;
    margin-bottom: 20px;
  }

  .note {
    color: var(--text-3);
    font-size: 0.875rem;
  }
  .note a { color: var(--lime); text-decoration: underline; }

  /* Friends Grid */
  .friend-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 8px;
    margin-bottom: 20px;
  }
  .friend-option {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 16px 8px;
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    background: var(--bg-surface);
    color: var(--text-2);
    cursor: pointer;
    transition: all var(--dur-fast) var(--ease-out);
  }
  .friend-option:hover { border-color: var(--text-3); }
  .friend-option.selected {
    border-color: var(--lime);
    background: var(--lime-dim);
    color: var(--text-1);
  }
  .f-name { font-weight: 600; font-size: 0.8125rem; }
  .f-handle { font-size: 0.6875rem; color: var(--text-3); }

  /* Events */
  .event-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 20px;
  }
  .event-option {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    background: var(--bg-surface);
    color: var(--text-2);
    cursor: pointer;
    text-align: left;
    transition: all var(--dur-fast) var(--ease-out);
  }
  .event-option:hover { border-color: var(--text-3); }
  .event-option.selected {
    border-color: var(--lime);
    background: var(--lime-dim);
    color: var(--text-1);
  }
  .ev-cat {
    font-size: 0.6875rem;
    text-transform: uppercase;
    font-weight: 700;
    color: var(--sky);
    min-width: 56px;
  }
  .ev-title { flex: 1; font-weight: 500; font-size: 0.875rem; }
  .ev-time { font-size: 0.75rem; color: var(--text-3); }

  /* Form */
  .form-group {
    margin-bottom: 16px;
  }
  .form-group label {
    display: block;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-3);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-bottom: 6px;
  }
  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }
  .hint {
    font-size: 0.75rem;
    color: var(--text-3);
    margin-top: 4px;
    display: block;
    font-family: var(--font-mono);
  }
  .odds-input {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .odds-input .input { width: 64px; text-align: center; }

  /* Review */
  .review-card {
    background: var(--bg-raised);
    border-radius: var(--r-md);
    padding: 16px;
    margin-bottom: 20px;
  }
  .review-row {
    display: flex;
    justify-content: space-between;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
    font-size: 0.875rem;
  }
  .review-row:last-child { border-bottom: none; }
  .review-label { color: var(--text-3); }
  .review-val { font-weight: 600; color: var(--text-1); }

  .step-actions {
    display: flex;
    justify-content: space-between;
    margin-top: 20px;
  }

  .error {
    color: var(--rose);
    font-size: 0.875rem;
    margin-top: 12px;
  }
</style>
