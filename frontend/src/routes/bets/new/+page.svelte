<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { coinBalance } from '$lib/stores';
  import { runWhenAuthResolved } from '$lib/auth';
  import { getFriends, getEvents, createBet, type FriendSummary } from '$lib/api';
  import { Globe, Zap, ArrowLeft, ArrowRight, Send } from 'lucide-svelte';

  type EventSummary = {
    id: string;
    title: string;
    category: string;
    sport?: string | null;
    league?: string | null;
    starts_at?: string | null;
    status?: string;
  };

  type SideOption = { label: string; value: string };

  let step = $state(1);
  let friends = $state<FriendSummary[]>([]);
  let allEvents = $state<EventSummary[]>([]);
  let loading = $state(true);
  let submitting = $state(false);
  let error = $state('');
  let selectedSide = $state('');
  let preferredOpponentId = '';
  let preferredEventId = '';
  let activeLeague = $state('all');

  let form = $state({
    opponent_id: '' as string | null,
    event_id: '',
    question: '',
    creator_position: '',
    opponent_position: '',
    amount: 50,
    odds_numerator: 1,
    odds_denominator: 1,
    expires_in_hours: 24,
  });

  let selectedFriend = $state<FriendSummary | null>(null);
  let selectedEvent = $state<EventSummary | null>(null);

  function leagues(): string[] {
    const set = new Set(allEvents.map((e) => (e.league || e.category || 'other').toUpperCase()));
    return Array.from(set).sort();
  }

  function filteredEvents(): EventSummary[] {
    if (activeLeague === 'all') return allEvents;
    return allEvents.filter((e) => (e.league || e.category || '').toUpperCase() === activeLeague);
  }

  function parseSides(title: string): SideOption[] {
    const patterns = [/@/, /\bvs\.?\b/i];
    for (const pattern of patterns) {
      const parts = title.split(pattern).map((p) => p.trim()).filter(Boolean);
      if (parts.length === 2) return parts.map((p) => ({ label: p, value: p }));
    }
    return [];
  }

  function sideOptions() {
    return selectedEvent ? parseSides(selectedEvent.title) : [];
  }

  function applySelectedSide(choice: string) {
    selectedSide = choice;
    form.creator_position = choice;
    const opposite = sideOptions().find((o) => o.value !== choice);
    form.opponent_position = opposite?.value || '';
  }

  function applyPreferredOpponent() {
    if (!preferredOpponentId) return;
    const match = friends.find((f) => f.user_id === preferredOpponentId);
    if (match) selectFriend(match);
  }

  function applyPreferredEvent() {
    if (!preferredEventId) return;
    const match = allEvents.find((event) => event.id === preferredEventId);
    if (!match) return;
    selectEvent(match);
    step = 2;
  }

  onMount(() => {
    const params = new URLSearchParams(window.location.search);
    preferredOpponentId = params.get('opponent') || '';
    preferredEventId = params.get('event') || '';
    return runWhenAuthResolved(async () => {
      loading = true;
      try {
        const [f, e] = await Promise.all([
          getFriends().catch(() => []),
          getEvents({ status: 'upcoming' }).catch(() => []),
        ]);
        friends = f;
        allEvents = e;
        applyPreferredOpponent();
        applyPreferredEvent();
      } catch {
        friends = [];
        allEvents = [];
      } finally {
        loading = false;
      }
    }, async () => {
      friends = [];
      allEvents = [];
      loading = false;
    });
  });

  function selectFriend(friend: FriendSummary) {
    selectedFriend = friend;
    form.opponent_id = friend.user_id;
  }

  function selectOpen() {
    selectedFriend = null;
    form.opponent_id = null;
  }

  function selectEvent(event: EventSummary) {
    selectedEvent = event;
    form.event_id = event.id;
    form.question = event.title;
    selectedSide = '';
    form.creator_position = '';
    form.opponent_position = '';

    const [firstSide] = parseSides(event.title);
    if (firstSide) applySelectedSide(firstSide.value);
  }

  async function submit() {
    if (submitting) return;
    error = '';
    if (form.opponent_id !== null && !form.opponent_id) { error = 'Select an opponent or post publicly'; return; }
    if (!form.event_id) { error = 'Select an event'; return; }
    if (!form.creator_position || !form.opponent_position) { error = 'Choose a side'; return; }
    if (form.amount <= 0) { error = 'Amount must be positive'; return; }
    if (form.amount > $coinBalance) { error = 'Insufficient coins'; return; }

    submitting = true;
    try {
      const payload: Record<string, any> = {
        event_id: form.event_id,
        question: form.question,
        creator_position: form.creator_position,
        opponent_position: form.opponent_position,
        amount: form.amount,
        odds_numerator: 1,
        odds_denominator: 1,
        expires_in_hours: form.expires_in_hours,
      };
      if (form.opponent_id !== null) payload.opponent_id = form.opponent_id;
      const bet = await createBet(payload);
      coinBalance.update((b) => b - form.amount);
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

<div class="max-w-[580px]">
  <h1 class="text-2xl font-display tracking-tight mb-6">Create a Bet</h1>

  <!-- Step indicators -->
  <div class="flex items-center mb-8">
    {#each [1,2,3,4] as s}
      <div class="w-3 h-3 rounded-full transition-all duration-200 shrink-0
        {step >= s ? 'bg-lime border-2 border-lime' : 'bg-raised border-2 border-border'}
        {step === s ? 'shadow-[0_0_0_4px_var(--color-lime-dim)]' : ''}"></div>
      {#if s < 4}<div class="flex-1 h-0.5 transition-all duration-300 {step > s ? 'bg-lime' : 'bg-border'}"></div>{/if}
    {/each}
  </div>

  <!-- STEP 1: Opponent -->
  {#if step === 1}
    <div class="bg-surface border border-border rounded-lg p-7 animate-enter">
      <h2 class="text-lg font-display font-bold mb-5">Who are you betting?</h2>

      <button class="flex items-center gap-3.5 w-full p-4 border rounded-md text-left cursor-pointer transition-all duration-150 mb-4
        {form.opponent_id === null ? 'border-lime bg-lime-dim text-text-1' : 'border-border bg-surface text-text-2 hover:border-border-strong'}"
        onclick={() => selectOpen()}>
        <Globe size={20} class="text-text-3" />
        <div>
          <span class="block font-semibold text-[0.8125rem]">Post Publicly</span>
          <span class="block text-[0.6875rem] text-text-3">Anyone on SideBet can take this bet</span>
        </div>
      </button>

      {#if loading}
        <p class="text-text-3 text-sm">Loading friends…</p>
      {:else if friends.length === 0}
        <p class="text-text-3 text-sm">No friends yet. Post publicly or <a href="/friends" class="text-lime underline">add friends</a>.</p>
      {:else}
        <p class="text-center text-text-3 text-xs my-3 font-medium">— or challenge a friend —</p>
        <div class="grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-2 mb-5">
          {#each friends as friend}
            <button class="flex flex-col items-center gap-1.5 p-4 border rounded-md cursor-pointer transition-all duration-150 min-h-11
              {form.opponent_id === friend.user_id ? 'border-lime bg-lime-dim text-text-1' : 'border-border bg-surface text-text-2 hover:border-border-strong'}"
              onclick={() => selectFriend(friend)}>
              <span class="w-8 h-8 rounded-full flex items-center justify-center font-display font-bold text-xs text-white bg-lime">{friend.display_name?.[0]?.toUpperCase() || '?'}</span>
              <span class="font-semibold text-[0.8125rem]">{friend.display_name || friend.username}</span>
              <span class="text-[0.6875rem] text-text-3">@{friend.username}</span>
            </button>
          {/each}
        </div>
      {/if}

      <button class="inline-flex items-center justify-center gap-1.5 px-5 py-2.5 font-display text-sm font-semibold bg-lime text-white rounded-md hover:bg-lime-hover active:scale-95 transition-all duration-150 cursor-pointer border-none disabled:opacity-40 disabled:cursor-not-allowed"
        disabled={form.opponent_id !== null && !form.opponent_id}
        onclick={() => step = 2}>
        Continue
        <ArrowRight size={16} />
      </button>
    </div>

  <!-- STEP 2: Event -->
  {:else if step === 2}
    <div class="bg-surface border border-border rounded-lg p-7 animate-enter">
      <h2 class="text-lg font-display font-bold mb-5">What event?</h2>

      {#if leagues().length > 1}
        <div class="flex flex-wrap gap-1.5 mb-4">
          <button class="px-3 py-1 text-xs font-semibold rounded-full border transition-all duration-150 cursor-pointer
            {activeLeague === 'all' ? 'bg-lime text-white border-lime' : 'bg-transparent text-text-2 border-border hover:border-border-strong'}"
            onclick={() => activeLeague = 'all'}>All</button>
          {#each leagues() as league}
            <button class="px-3 py-1 text-xs font-semibold rounded-full border transition-all duration-150 cursor-pointer
              {activeLeague === league ? 'bg-lime text-white border-lime' : 'bg-transparent text-text-2 border-border hover:border-border-strong'}"
              onclick={() => activeLeague = league}>{league}</button>
          {/each}
        </div>
      {/if}

      {#if allEvents.length === 0 && !loading}
        <p class="text-text-3 text-sm">No upcoming events. Try syncing on the <a href="/events" class="text-lime underline">Events</a> page.</p>
      {:else if filteredEvents().length === 0}
        <p class="text-text-3 text-sm">No events in this league.</p>
      {:else}
        <div class="flex flex-col gap-1.5 mb-5 max-h-[400px] overflow-y-auto">
          {#each filteredEvents() as event}
            <button class="flex items-center gap-3 p-3 px-3.5 border rounded-md text-left cursor-pointer transition-all duration-150 min-h-11
              {form.event_id === event.id ? 'border-lime bg-lime-dim text-text-1' : 'border-border bg-surface text-text-2 hover:border-border-strong'}"
              onclick={() => selectEvent(event)}>
              <span class="text-[0.6875rem] uppercase font-bold text-sky min-w-14">{(event.league || event.category || '').toUpperCase()}</span>
              <span class="flex-1 font-medium text-sm">{event.title}</span>
              {#if event.starts_at}
                <span class="text-xs text-text-3 shrink-0">{new Date(event.starts_at).toLocaleDateString(undefined, { month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit' })}</span>
              {/if}
            </button>
          {/each}
        </div>
        <div class="flex justify-between mt-5">
          <button class="inline-flex items-center justify-center gap-1.5 px-5 py-2.5 font-display text-sm font-semibold bg-transparent text-text-2 border border-border rounded-md hover:bg-raised hover:text-text-1 active:scale-95 transition-all duration-150 cursor-pointer"
            onclick={() => step = 1}>
            <ArrowLeft size={16} />
            Back
          </button>
          <button class="inline-flex items-center justify-center gap-1.5 px-5 py-2.5 font-display text-sm font-semibold bg-lime text-white rounded-md hover:bg-lime-hover active:scale-95 transition-all duration-150 cursor-pointer border-none disabled:opacity-40 disabled:cursor-not-allowed"
            disabled={!form.event_id}
            onclick={() => step = 3}>
            Continue
            <ArrowRight size={16} />
          </button>
        </div>
      {/if}
    </div>

  <!-- STEP 3: Details -->
  {:else if step === 3}
    <div class="bg-surface border border-border rounded-lg p-7 animate-enter">
      <h2 class="text-lg font-display font-bold mb-5">Bet details</h2>

      <div class="mb-5">
        <label for="bet-side" class="block text-xs font-semibold text-text-3 uppercase tracking-wide mb-1.5">Pick a side</label>
        <select id="bet-side" bind:value={selectedSide}
          onchange={(e) => applySelectedSide((e.currentTarget as HTMLSelectElement).value)}
          class="w-full py-2.5 px-3.5 font-body text-[0.9375rem] bg-input text-text-1 border border-border rounded-md outline-none transition-all duration-150 focus:border-lime focus:shadow-[0_0_0_2px_var(--color-lime-dim)]">
          <option value="" disabled selected={!selectedSide}>Choose the side you want</option>
          {#each sideOptions() as option}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
        {#if form.opponent_position}
          <span class="text-xs text-text-3 mt-1 block">Opponent takes: <strong class="text-text-2">{form.opponent_position}</strong></span>
        {/if}
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div class="mb-4">
          <label for="bet-amount" class="block text-xs font-semibold text-text-3 uppercase tracking-wide mb-1.5">Wager (coins)</label>
          <input id="bet-amount" type="number" bind:value={form.amount} min="1" max={$coinBalance}
            class="w-full py-2.5 px-3.5 font-body text-[0.9375rem] bg-input text-text-1 border border-border rounded-md outline-none transition-all duration-150 focus:border-lime focus:shadow-[0_0_0_2px_var(--color-lime-dim)]" />
          <span class="text-xs text-text-3 mt-1 block font-mono">Balance: {$coinBalance.toLocaleString()} coins</span>
        </div>
        <div class="mb-4">
          <p class="block text-xs font-semibold text-text-3 uppercase tracking-wide mb-1.5">Odds</p>
          <div class="w-full py-2.5 px-3.5 bg-raised border border-border rounded-md font-mono tabular-nums text-[0.9375rem] text-text-1">Straight · 1:1</div>
          <span class="text-xs text-text-3 mt-1 block">Even money — winner takes all</span>
        </div>
      </div>

      {#if error}
        <p class="text-rose text-sm mt-3">{error}</p>
      {/if}

      <div class="flex justify-between mt-5">
        <button class="inline-flex items-center justify-center gap-1.5 px-5 py-2.5 font-display text-sm font-semibold bg-transparent text-text-2 border border-border rounded-md hover:bg-raised hover:text-text-1 active:scale-95 transition-all duration-150 cursor-pointer"
          onclick={() => step = 2}>
          <ArrowLeft size={16} />
          Back
        </button>
        <button class="inline-flex items-center justify-center gap-1.5 px-5 py-2.5 font-display text-sm font-semibold bg-lime text-white rounded-md hover:bg-lime-hover active:scale-95 transition-all duration-150 cursor-pointer border-none disabled:opacity-40 disabled:cursor-not-allowed"
          disabled={!form.creator_position}
          onclick={() => step = 4}>
          Review
          <ArrowRight size={16} />
        </button>
      </div>
    </div>

  <!-- STEP 4: Review -->
  {:else if step === 4}
    <div class="bg-surface border border-border rounded-lg p-7 animate-enter">
      <h2 class="text-lg font-display font-bold mb-5">Review your bet</h2>
      <div class="bg-raised rounded-md p-4 mb-5">
        {#each [
          ['Opponent', selectedFriend ? (selectedFriend.display_name || selectedFriend.username) : '🌐 Open to everyone'],
          ['Event', selectedEvent?.title],
          ['Your pick', form.creator_position],
          ['Their pick', form.opponent_position],
          ['Wager', `${form.amount.toLocaleString()} coins`],
          ['Odds', 'Straight · 1:1'],
          ['Payout', `${(form.amount * 2).toLocaleString()} coins`],
        ] as [label, value]}
          <div class="flex justify-between py-2 border-b border-border last:border-b-0 text-sm">
            <span class="text-text-3">{label}</span>
            <span class="font-semibold text-text-1
              {label === 'Wager' || label === 'Odds' || label === 'Payout' ? 'font-mono tabular-nums' : ''}
              {label === 'Payout' ? 'text-lime' : ''}">{value}</span>
          </div>
        {/each}
      </div>

      {#if error}
        <p class="text-rose text-sm mt-3">{error}</p>
      {/if}

      <div class="flex justify-between mt-5">
        <button class="inline-flex items-center justify-center gap-1.5 px-5 py-2.5 font-display text-sm font-semibold bg-transparent text-text-2 border border-border rounded-md hover:bg-raised hover:text-text-1 active:scale-95 transition-all duration-150 cursor-pointer"
          onclick={() => step = 3}>
          <ArrowLeft size={16} />
          Back
        </button>
        <button class="inline-flex items-center justify-center gap-1.5 px-5 py-2.5 font-display text-sm font-semibold bg-lime text-white rounded-md hover:bg-lime-hover active:scale-95 transition-all duration-150 cursor-pointer border-none disabled:opacity-40 disabled:cursor-not-allowed"
          onclick={submit} disabled={submitting}>
          <Send size={16} />
          {submitting ? 'Sending…' : 'Send Bet'}
        </button>
      </div>
    </div>
  {/if}
</div>
