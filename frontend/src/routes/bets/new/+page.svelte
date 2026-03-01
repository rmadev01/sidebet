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

  const friends = [
    { id: 'f1', username: 'chad_bets', name: 'Chad', wins: 15, losses: 8, av: 1 },
    { id: 'f2', username: 'crypto_mike', name: 'Mike', wins: 22, losses: 12, av: 2 },
    { id: 'f3', username: 'hoops_fan', name: 'Sarah', wins: 9, losses: 5, av: 3 },
    { id: 'f4', username: 'poly_whale', name: 'Alex', wins: 18, losses: 14, av: 4 },
  ];

  function impliedProb(n: number, d: number) {
    return ((d / (n + d)) * 100).toFixed(1) + '%';
  }
</script>

<svelte:head>
  <title>SideBet — New Bet</title>
</svelte:head>

<div class="create">
  <h1 class="animate-in">Create a Bet</h1>
  <p class="subtitle animate-in" style="animation-delay:40ms">Challenge a friend to a wager</p>

  <!-- Step indicator — fading numbers -->
  <div class="step-row animate-in" style="animation-delay:60ms">
    <div class="step-item" class:active={step >= 1} class:done={step > 1}><span class="step-n">{step > 1 ? '✓' : '1'}</span> Question</div>
    <div class="step-line" class:done={step > 1}></div>
    <div class="step-item" class:active={step >= 2} class:done={step > 2}><span class="step-n">{step > 2 ? '✓' : '2'}</span> Terms</div>
    <div class="step-line" class:done={step > 2}></div>
    <div class="step-item" class:active={step >= 3}><span class="step-n">3</span> Send</div>
  </div>

  <div class="form-area animate-in" style="animation-delay:100ms">
    {#if step === 1}
      <!-- Step 1: Question -->
      <div class="form-card">
        <div class="field">
          <label class="label">Bet Type</label>
          <div class="type-toggle">
            <button class="type-btn" class:active={betType === 'event'} onclick={() => betType = 'event'}>From an Event</button>
            <button class="type-btn" class:active={betType === 'custom'} onclick={() => betType = 'custom'}>Custom</button>
          </div>
        </div>

        <div class="field">
          <label class="label" for="q">Question / Assertion</label>
          <textarea id="q" class="input" bind:value={question} placeholder="e.g. The Lakers will defeat the Celtics on March 5, 2026."></textarea>
          <span class="hint">This will be the UMA assertion claim. Be specific.</span>
        </div>

        <div class="field-row">
          <div class="field">
            <label class="label" for="yp">Your Position</label>
            <input id="yp" class="input" bind:value={creatorPosition} placeholder="e.g. Lakers win" />
          </div>
          <div class="field">
            <label class="label" for="tp">Their Position</label>
            <input id="tp" class="input" bind:value={opponentPosition} placeholder="e.g. Celtics win" />
          </div>
        </div>

        <button class="btn btn-primary btn-lg full-w" onclick={() => step = 2} disabled={!question || !creatorPosition || !opponentPosition}>
          Continue
        </button>
      </div>

    {:else if step === 2}
      <!-- Step 2: Terms & Odds -->
      <div class="terms-grid">
        <div class="form-card">
          <div class="field-row">
            <div class="field">
              <label class="label" for="amt">Wager (ETH)</label>
              <input id="amt" class="input" type="number" step="0.001" min="0.001" bind:value={amountEth} />
            </div>
            <div class="field">
              <label class="label" for="exp">Expires in (hours)</label>
              <input id="exp" class="input" type="number" min="1" max="168" bind:value={expiresHours} />
            </div>
          </div>

          <div class="field">
            <label class="label">Your Odds</label>
            <div class="odds-row">
              <input class="input odds-in" type="number" min="1" bind:value={oddsNumerator} />
              <span class="odds-colon">:</span>
              <input class="input odds-in" type="number" min="1" bind:value={oddsDenominator} />
            </div>
            <span class="hint">Implied probability: {impliedProb(oddsNumerator, oddsDenominator)}</span>
          </div>

          <div class="btns-row">
            <button class="btn btn-ghost" onclick={() => step = 1}>Back</button>
            <button class="btn btn-primary btn-lg" style="flex:1" onclick={() => step = 3}>Choose Opponent</button>
          </div>
        </div>

        <!-- Odds comparison -->
        <div class="odds-panel">
          <h4>Reference Odds</h4>
          <div class="ref-item"><span class="ref-label">Sportsbook Avg</span><span class="mono ref-val">1.82</span></div>
          <div class="ref-item"><span class="ref-label">Polymarket</span><span class="mono ref-val">1.91</span></div>
          <div class="ref-item ref-yours"><span class="ref-label">Your odds</span><span class="mono ref-val">{((oddsNumerator + oddsDenominator) / oddsDenominator).toFixed(2)}</span></div>
          <div class="ref-delta">
            <span>Delta from market</span>
            <span class="mono" style="color:var(--amber)">+8.2%</span>
          </div>
        </div>
      </div>

    {:else}
      <!-- Step 3: Select Opponent -->
      <div class="form-card">
        <div class="field">
          <label class="label">Select Opponent</label>
          <div class="friend-scroll">
            {#each friends as f}
              <button class="f-chip" class:selected={selectedFriend === f.id} onclick={() => selectedFriend = f.id}>
                <div class="av av-{f.av}">{f.name[0]}</div>
                <span class="f-name">@{f.username}</span>
                <span class="f-record">{f.wins}W–{f.losses}L</span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Receipt-style summary -->
        <div class="receipt">
          <div class="receipt-title">Summary</div>
          <div class="receipt-row"><span>Question</span><span class="receipt-val">{question.length > 40 ? question.slice(0, 40) + '…' : question}</span></div>
          <div class="receipt-row"><span>Your pick</span><span class="receipt-val">{creatorPosition}</span></div>
          <div class="receipt-row"><span>Amount</span><span class="receipt-val mono">{amountEth} ETH</span></div>
          <div class="receipt-row"><span>Odds</span><span class="receipt-val">{oddsNumerator}:{oddsDenominator}</span></div>
          <div class="receipt-row"><span>Expires</span><span class="receipt-val">{expiresHours}h</span></div>
        </div>

        <div class="btns-row">
          <button class="btn btn-ghost" onclick={() => step = 2}>Back</button>
          <button class="btn btn-primary btn-lg" style="flex:1" disabled={!selectedFriend}>Send Bet</button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .create { max-width: 860px; }
  .create h1 { margin-bottom: 4px; }
  .subtitle { color: var(--text-2); font-size: 0.9375rem; margin-bottom: 28px; }

  /* ── Steps ── */
  .step-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0;
    margin-bottom: 32px;
  }
  .step-item {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: var(--font-display);
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-3);
    transition: color var(--dur-base);
  }
  .step-item.active { color: var(--text-1); }
  .step-n {
    width: 24px;
    height: 24px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    font-size: 0.6875rem;
    font-weight: 700;
    background: var(--bg-raised);
    border: 1px solid var(--border);
  }
  .step-item.active .step-n {
    background: var(--lime);
    border-color: var(--lime);
    color: var(--bg-root);
  }
  .step-item.done .step-n {
    background: var(--lime-dim);
    border-color: var(--lime);
    color: var(--lime);
  }
  .step-line {
    width: 48px;
    height: 1px;
    background: var(--border);
    margin: 0 8px;
  }
  .step-line.done { background: var(--lime); }

  /* ── Form Card ── */
  .form-card {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    padding: 28px;
    max-width: 560px;
    margin: 0 auto;
  }

  .field { margin-bottom: 20px; }
  .field-row { display: flex; gap: 14px; margin-bottom: 20px; }
  .field-row .field { flex: 1; margin-bottom: 0; }
  .hint { font-size: 0.6875rem; color: var(--text-3); margin-top: 4px; display: block; }
  .full-w { width: 100%; }

  .type-toggle { display: flex; gap: 6px; }
  .type-btn {
    flex: 1;
    padding: 10px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    color: var(--text-3);
    font-family: var(--font-display);
    font-size: 0.8125rem;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--dur-fast);
  }
  .type-btn.active {
    border-color: var(--lime);
    background: var(--lime-dim);
    color: var(--text-1);
  }

  .odds-row { display: flex; align-items: center; gap: 8px; }
  .odds-in { width: 72px; text-align: center; }
  .odds-colon { font-size: 1.125rem; font-weight: 800; color: var(--text-3); }

  .btns-row { display: flex; gap: 10px; margin-top: 24px; }

  /* ── Terms Grid ── */
  .terms-grid {
    display: flex;
    gap: 20px;
    align-items: flex-start;
  }
  .terms-grid .form-card { flex: 1; max-width: none; }

  /* ── Odds Panel ── */
  .odds-panel {
    width: 240px;
    flex-shrink: 0;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    padding: 20px;
    position: sticky;
    top: calc(var(--nav-height) + 16px);
  }
  .odds-panel h4 { margin-bottom: 14px; font-size: 0.875rem; }
  .ref-item {
    display: flex;
    justify-content: space-between;
    padding: 6px 0;
    border-bottom: 1px solid var(--border);
    font-size: 0.8125rem;
  }
  .ref-label { color: var(--text-3); }
  .ref-val { font-weight: 600; font-size: 0.8125rem; }
  .ref-yours { color: var(--lime); }
  .ref-delta {
    display: flex;
    justify-content: space-between;
    padding-top: 10px;
    font-size: 0.8125rem;
    color: var(--text-3);
  }

  /* ── Friend Selector ── */
  .friend-scroll {
    display: flex;
    gap: 8px;
    overflow-x: auto;
    padding: 4px 0;
    -webkit-overflow-scrolling: touch;
  }
  .f-chip {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 14px 16px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    cursor: pointer;
    transition: all var(--dur-fast);
    flex-shrink: 0;
    min-width: 100px;
    font-family: var(--font-body);
  }
  .f-chip:hover { border-color: var(--border-strong); }
  .f-chip.selected {
    border-color: var(--lime);
    background: var(--lime-dim);
  }
  .f-name { font-size: 0.75rem; font-weight: 600; color: var(--text-1); }
  .f-record { font-size: 0.6875rem; color: var(--text-3); }

  /* ── Receipt Summary ── */
  .receipt {
    margin-top: 20px;
    border: 1px dashed var(--border-strong);
    border-radius: var(--r-md);
    padding: 16px;
  }
  .receipt-title {
    font-family: var(--font-display);
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-3);
    margin-bottom: 10px;
  }
  .receipt-row {
    display: flex;
    justify-content: space-between;
    padding: 4px 0;
    font-size: 0.8125rem;
    color: var(--text-2);
  }
  .receipt-val { color: var(--text-1); font-weight: 500; }

  @media (max-width: 720px) {
    .terms-grid { flex-direction: column; }
    .odds-panel { width: 100%; position: static; }
    .field-row { flex-direction: column; }
    .friend-scroll { flex-wrap: wrap; }
    .f-chip { min-width: 0; flex: 1; }
  }
</style>
