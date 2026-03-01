<script lang="ts">
  let step = $state(1);
  let betType = $state<'event' | 'custom'>('event');
  let question = $state('');
  let creatorPosition = $state('');
  let opponentPosition = $state('');
  let amountEth = $state('0.1');
  let oddsNumerator = $state(1);
  let oddsDenominator = $state(1);
  let selectedFriend = $state('');
  let expiresHours = $state(24);

  const demoFriends = [
    { id: 'f1', username: 'chad_bets', display_name: 'Chad', wins: 15, losses: 8 },
    { id: 'f2', username: 'crypto_mike', display_name: 'Mike', wins: 22, losses: 12 },
    { id: 'f3', username: 'hoops_fan', display_name: 'Sarah', wins: 9, losses: 5 },
  ];

  function impliedProb(n: number, d: number): string {
    return ((d / (n + d)) * 100).toFixed(1) + '%';
  }

  function marketProb(): string {
    return '52.3%';
  }
</script>

<svelte:head>
  <title>SideBet — New Bet</title>
</svelte:head>

<div class="new-bet-page">
  <div class="page-header animate-slide-up">
    <h1>Create a Bet</h1>
    <p class="text-secondary">Challenge a friend to a wager</p>
  </div>

  <!-- Progress Steps -->
  <div class="steps animate-slide-up" style="animation-delay: 60ms">
    <div class="step" class:active={step >= 1} class:done={step > 1}>
      <span class="step-num">{step > 1 ? '✓' : '1'}</span>
      <span class="step-label">Question</span>
    </div>
    <div class="step-line" class:active={step > 1}></div>
    <div class="step" class:active={step >= 2} class:done={step > 2}>
      <span class="step-num">{step > 2 ? '✓' : '2'}</span>
      <span class="step-label">Terms</span>
    </div>
    <div class="step-line" class:active={step > 2}></div>
    <div class="step" class:active={step >= 3}>
      <span class="step-num">3</span>
      <span class="step-label">Send</span>
    </div>
  </div>

  <div class="form-area animate-slide-up" style="animation-delay: 120ms">
    {#if step === 1}
      <!-- Step 1: Question -->
      <div class="card-flat form-card">
        <div class="form-group">
          <label class="label">Bet Type</label>
          <div class="toggle-row">
            <button class="toggle-btn" class:active={betType === 'event'} onclick={() => betType = 'event'}>
              📅 From an Event
            </button>
            <button class="toggle-btn" class:active={betType === 'custom'} onclick={() => betType = 'custom'}>
              ✏️ Custom Question
            </button>
          </div>
        </div>

        <div class="form-group">
          <label class="label" for="question">Question / Assertion</label>
          <textarea
            id="question"
            class="input"
            bind:value={question}
            placeholder="e.g. The Los Angeles Lakers will defeat the Boston Celtics on March 5, 2026."
          ></textarea>
          <span class="text-xs text-muted">This will be the UMA assertion claim. Be specific and unambiguous.</span>
        </div>

        <div class="form-row">
          <div class="form-group" style="flex:1">
            <label class="label" for="your-position">Your Position</label>
            <input id="your-position" class="input" bind:value={creatorPosition} placeholder="e.g. Lakers win" />
          </div>
          <div class="form-group" style="flex:1">
            <label class="label" for="their-position">Their Position</label>
            <input id="their-position" class="input" bind:value={opponentPosition} placeholder="e.g. Celtics win" />
          </div>
        </div>

        <button class="btn btn-primary btn-lg" style="width:100%" onclick={() => step = 2} disabled={!question || !creatorPosition || !opponentPosition}>
          Continue →
        </button>
      </div>

    {:else if step === 2}
      <!-- Step 2: Terms & Odds -->
      <div class="terms-layout">
        <div class="card-flat form-card">
          <div class="form-row">
            <div class="form-group" style="flex:1">
              <label class="label" for="amount">Wager Amount (ETH)</label>
              <input id="amount" class="input" type="number" step="0.001" min="0.001" bind:value={amountEth} />
            </div>
            <div class="form-group" style="flex:1">
              <label class="label" for="expires">Expires In (hours)</label>
              <input id="expires" class="input" type="number" min="1" max="168" bind:value={expiresHours} />
            </div>
          </div>

          <div class="form-group">
            <label class="label">Your Odds</label>
            <div class="odds-input-row">
              <input class="input odds-input" type="number" min="1" bind:value={oddsNumerator} />
              <span class="odds-colon">:</span>
              <input class="input odds-input" type="number" min="1" bind:value={oddsDenominator} />
            </div>
            <span class="text-xs text-muted">Your implied probability: {impliedProb(oddsNumerator, oddsDenominator)}</span>
          </div>

          <div class="form-actions">
            <button class="btn btn-ghost" onclick={() => step = 1}>← Back</button>
            <button class="btn btn-primary btn-lg" style="flex:1" onclick={() => step = 3}>
              Choose Opponent →
            </button>
          </div>
        </div>

        <!-- Reference Odds Panel -->
        <div class="card-flat odds-panel">
          <h4>📊 Reference Odds</h4>
          <div class="ref-row">
            <span class="ref-label">Sportsbook Avg</span>
            <span class="ref-value">1.82 ({marketProb()})</span>
          </div>
          <div class="ref-row">
            <span class="ref-label">Polymarket</span>
            <span class="ref-value">1.91 ({marketProb()})</span>
          </div>
          <div class="ref-row highlight">
            <span class="ref-label">Your Odds</span>
            <span class="ref-value">{((oddsNumerator + oddsDenominator) / oddsDenominator).toFixed(2)} ({impliedProb(oddsNumerator, oddsDenominator)})</span>
          </div>
          <div class="delta-indicator">
            <span class="delta-label">Delta from market</span>
            <span class="delta-value" style="color: var(--accent-gold)">+8.2%</span>
          </div>
        </div>
      </div>

    {:else}
      <!-- Step 3: Select Opponent -->
      <div class="card-flat form-card">
        <div class="form-group">
          <label class="label">Select Opponent</label>
          <div class="friends-select">
            {#each demoFriends as friend}
              <button
                class="friend-option card"
                class:selected={selectedFriend === friend.id}
                onclick={() => selectedFriend = friend.id}
              >
                <div class="avatar">{friend.display_name[0]}</div>
                <div class="friend-option-info">
                  <span class="friend-option-name">@{friend.username}</span>
                  <span class="text-xs text-muted">{friend.wins}W - {friend.losses}L</span>
                </div>
                {#if selectedFriend === friend.id}
                  <span style="color: var(--accent-green)">✓</span>
                {/if}
              </button>
            {/each}
          </div>
        </div>

        <!-- Summary -->
        <div class="bet-summary">
          <h4>Bet Summary</h4>
          <div class="summary-row"><span>Question:</span> <span>{question}</span></div>
          <div class="summary-row"><span>Your Pick:</span> <span>{creatorPosition}</span></div>
          <div class="summary-row"><span>Amount:</span> <span class="bet-row-amount">{amountEth} ETH</span></div>
          <div class="summary-row"><span>Odds:</span> <span>{oddsNumerator}:{oddsDenominator}</span></div>
          <div class="summary-row"><span>Expires:</span> <span>{expiresHours}h</span></div>
        </div>

        <div class="form-actions">
          <button class="btn btn-ghost" onclick={() => step = 2}>← Back</button>
          <button class="btn btn-success btn-lg" style="flex:1" disabled={!selectedFriend}>
            🎲 Send Bet
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .new-bet-page { max-width: 900px; }
  .page-header { margin-bottom: var(--space-lg); }
  .page-header h1 { margin-bottom: 4px; }

  /* Steps */
  .steps {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0;
    margin-bottom: var(--space-2xl);
  }
  .step {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    opacity: 0.4;
    transition: opacity var(--transition-base);
  }
  .step.active { opacity: 1; }
  .step-num {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: var(--bg-card);
    font-size: 0.8125rem;
    font-weight: 700;
    border: 1px solid var(--border-color);
  }
  .step.active .step-num {
    background: var(--accent-blue);
    border-color: var(--accent-blue);
    color: white;
  }
  .step.done .step-num {
    background: var(--accent-green);
    border-color: var(--accent-green);
    color: white;
  }
  .step-label { font-size: 0.875rem; font-weight: 500; }
  .step-line {
    width: 60px;
    height: 2px;
    background: var(--border-color);
    margin: 0 var(--space-sm);
    transition: background var(--transition-base);
  }
  .step-line.active { background: var(--accent-blue); }

  /* Form */
  .form-card { max-width: 600px; margin: 0 auto; }
  .form-group { margin-bottom: var(--space-lg); }
  .form-row { display: flex; gap: var(--space-md); }
  .form-actions { display: flex; gap: var(--space-md); margin-top: var(--space-lg); }

  .toggle-row { display: flex; gap: var(--space-sm); }
  .toggle-btn {
    flex: 1;
    padding: var(--space-md);
    background: var(--bg-input);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 0.875rem;
    cursor: pointer;
    transition: all var(--transition-fast);
  }
  .toggle-btn.active {
    border-color: var(--accent-blue);
    background: var(--accent-blue-dim);
    color: var(--text-primary);
  }

  .odds-input-row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }
  .odds-input { width: 80px; text-align: center; }
  .odds-colon {
    font-size: 1.25rem;
    font-weight: 800;
    color: var(--text-muted);
  }

  /* Terms layout */
  .terms-layout {
    display: flex;
    gap: var(--space-xl);
    align-items: flex-start;
  }
  .terms-layout .form-card { flex: 1; }

  /* Odds Panel */
  .odds-panel {
    width: 280px;
    flex-shrink: 0;
    position: sticky;
    top: var(--space-xl);
  }
  .odds-panel h4 { margin-bottom: var(--space-md); }
  .ref-row {
    display: flex;
    justify-content: space-between;
    padding: var(--space-sm) 0;
    border-bottom: 1px solid var(--border-subtle);
    font-size: 0.875rem;
  }
  .ref-label { color: var(--text-muted); }
  .ref-value { font-weight: 600; font-family: var(--font-mono); }
  .ref-row.highlight { color: var(--accent-blue); }
  .delta-indicator {
    display: flex;
    justify-content: space-between;
    padding-top: var(--space-md);
    font-size: 0.875rem;
  }
  .delta-label { color: var(--text-muted); }
  .delta-value { font-weight: 700; }

  /* Friend select */
  .friends-select { display: flex; flex-direction: column; gap: var(--space-sm); }
  .friend-option {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    cursor: pointer;
    border: 1px solid var(--border-subtle);
    text-align: left;
    font-family: var(--font-sans);
    padding: var(--space-md);
  }
  .friend-option.selected {
    border-color: var(--accent-green);
    background: var(--accent-green-dim);
  }
  .friend-option-info { flex: 1; display: flex; flex-direction: column; }
  .friend-option-name { font-weight: 600; font-size: 0.9375rem; }

  /* Summary */
  .bet-summary {
    background: var(--bg-input);
    border-radius: var(--radius-md);
    padding: var(--space-lg);
    margin-top: var(--space-lg);
  }
  .bet-summary h4 { margin-bottom: var(--space-md); }
  .summary-row {
    display: flex;
    justify-content: space-between;
    padding: var(--space-xs) 0;
    font-size: 0.875rem;
  }
  .summary-row span:first-child { color: var(--text-muted); }

  .bet-row-amount {
    font-weight: 700;
    font-family: var(--font-mono);
    color: var(--accent-blue);
  }

  @media (max-width: 768px) {
    .terms-layout { flex-direction: column; }
    .odds-panel { width: 100%; position: static; }
    .form-row { flex-direction: column; }
  }
</style>
