<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { coinBalance, isAuthenticated, user } from '$lib/stores';
  import { runWhenAuthResolved } from '$lib/auth';
  import { getBet, acceptBet, declineBet, cancelBet, settleBet, takeBet, type Bet } from '$lib/api';
  import { goto } from '$app/navigation';
  import { ArrowLeft, Trophy, Calendar, Check, X, Ban, Swords } from 'lucide-svelte';

  let bet = $state<Bet | null>(null);
  let loading = $state(true);
  let acting = $state(false);
  let error = $state('');

  onMount(() => {
    return runWhenAuthResolved(load, async () => {
      bet = null;
      loading = false;
      error = '';
    });
  });

  async function load() {
    loading = true;
    try {
      const id = $page.params.id;
      if (!id) { error = 'No bet ID'; loading = false; return; }
      bet = await getBet(id);
    } catch (e: any) {
      error = e.message;
    }
    loading = false;
  }

  function formatCoins(n: number) { return n?.toLocaleString() ?? '0'; }
  function isCreator() { return bet?.creator_id === $user?.id; }
  function isOpponent() { return bet?.opponent_id === $user?.id; }
  function canTakeOpenBet() { return !!bet && bet.status === 'open' && !isCreator(); }

  async function doAccept() {
    if (!bet) return;
    const current = bet;
    acting = true;
    error = '';
    try {
      bet = await acceptBet(current.id);
      coinBalance.update(b => b - current.amount);
    } catch (e: any) { error = e.message; }
    acting = false;
  }

  async function doDecline() {
    if (!bet) return;
    const current = bet;
    acting = true;
    error = '';
    try {
      await declineBet(current.id);
      bet = { ...current, status: 'declined' };
    } catch (e: any) { error = e.message; }
    acting = false;
  }

  async function doCancel() {
    if (!bet) return;
    const current = bet;
    acting = true;
    error = '';
    try {
      await cancelBet(current.id);
      bet = { ...current, status: 'cancelled' };
      coinBalance.update(b => b + current.amount);
    } catch (e: any) { error = e.message; }
    acting = false;
  }

  async function doSettle() {
    if (!bet) return;
    const current = bet;
    acting = true;
    error = '';
    try {
      bet = await settleBet(current.id, 'disputed');
    } catch (e: any) { error = e.message; }
    acting = false;
  }

  async function doTakeOpen() {
    if (!bet) return;
    acting = true;
    error = '';
    try {
      const updated = await takeBet(bet.id);
      coinBalance.update((b) => b - updated.amount);
      goto(`/bets/${updated.id}`);
    } catch (e: any) {
      error = e.message;
      acting = false;
    }
  }
</script>

<svelte:head>
  <title>Bet Details — SideBet</title>
</svelte:head>

<div class="max-w-2xl">
  {#if loading}
    <p class="text-text-3 text-sm py-10 text-center">Loading…</p>
  {:else if !$isAuthenticated}
    <div class="text-center py-10">
      <p class="text-text-3 mb-4">Sign in to view or respond to this bet.</p>
      <a href="/login?next=/bets/{$page.params.id}" class="inline-flex items-center justify-center px-5 py-2.5 font-display text-sm font-semibold bg-lime text-white rounded-md no-underline hover:bg-lime-hover active:scale-95 transition-all duration-150">Sign In</a>
    </div>
  {:else if !bet}
    <p class="text-text-3 text-sm py-10 text-center">Bet not found.</p>
  {:else}
    <a href="/bets" class="inline-flex items-center gap-1 mb-4 text-sm text-text-3 no-underline hover:text-text-2 transition-colors duration-150">
      <ArrowLeft size={14} />
      All Bets
    </a>

    <div class="bg-surface border border-border rounded-lg p-8 max-sm:p-5">
      <!-- Status Badge -->
      <div class="flex items-center justify-between mb-5">
        <span class="text-[0.6875rem] font-bold tracking-widest px-2.5 py-1 rounded-sm
          {bet.status === 'active' ? 'bg-sky-dim text-sky' : bet.status === 'proposed' ? 'bg-amber-dim text-amber' : bet.status === 'settled' ? 'bg-lime-dim text-lime' : bet.status === 'open' ? 'bg-sky-dim text-sky' : 'bg-raised text-text-3'}">{bet.status.toUpperCase()}</span>
        <span class="flex items-center gap-1 font-mono text-text-3 text-sm">
          <Calendar size={13} />
          {new Date(bet.created_at).toLocaleDateString()}
        </span>
      </div>

      <h1 class="text-[1.375rem] font-bold leading-snug mb-6 font-display tracking-tight">{bet.question}</h1>

      <!-- Positions -->
      <div class="flex items-center gap-4 mb-6 max-sm:flex-col">
        <div class="flex-1 bg-raised rounded-md p-4 text-center border-2 max-sm:w-full {bet.winner_id === bet.creator_id ? 'border-lime' : 'border-transparent'}">
          <span class="block text-[0.6875rem] font-semibold uppercase text-text-3 mb-1.5">Creator</span>
          <span class="block font-bold text-base text-text-1">{bet.creator_position}</span>
          {#if bet.winner_id === bet.creator_id}<span class="inline-flex items-center gap-1 mt-1.5 text-xs text-lime"><Trophy size={12} /> Winner</span>{/if}
        </div>
        <div class="text-xs text-text-3 font-bold">vs</div>
        <div class="flex-1 bg-raised rounded-md p-4 text-center border-2 max-sm:w-full {bet.winner_id === bet.opponent_id ? 'border-lime' : 'border-transparent'}">
          <span class="block text-[0.6875rem] font-semibold uppercase text-text-3 mb-1.5">Opponent</span>
          <span class="block font-bold text-base text-text-1">{bet.opponent_position}</span>
          {#if bet.winner_id === bet.opponent_id}<span class="inline-flex items-center gap-1 mt-1.5 text-xs text-lime"><Trophy size={12} /> Winner</span>{/if}
        </div>
      </div>

      <!-- Details Grid -->
      <div class="grid grid-cols-[repeat(auto-fit,minmax(140px,1fr))] gap-3 mb-6 p-4 bg-raised rounded-md">
        <div class="text-center">
          <span class="block text-[0.6875rem] font-semibold uppercase text-text-3 mb-1">Wager</span>
          <span class="font-bold text-[0.9375rem] text-text-1 font-mono tabular-nums">{formatCoins(bet.amount)} coins</span>
        </div>
        <div class="text-center">
          <span class="block text-[0.6875rem] font-semibold uppercase text-text-3 mb-1">Odds</span>
          <span class="font-bold text-[0.9375rem] text-text-1 font-mono tabular-nums">{bet.odds_numerator}:{bet.odds_denominator}</span>
        </div>
        <div class="text-center">
          <span class="block text-[0.6875rem] font-semibold uppercase text-text-3 mb-1">Expires</span>
          <span class="font-bold text-[0.9375rem] text-text-1">{new Date(bet.expires_at).toLocaleDateString()}</span>
        </div>
        {#if bet.resolved_at}
          <div class="text-center">
            <span class="block text-[0.6875rem] font-semibold uppercase text-text-3 mb-1">Settled</span>
            <span class="font-bold text-[0.9375rem] text-text-1">{new Date(bet.resolved_at).toLocaleDateString()}</span>
          </div>
        {/if}
      </div>

      {#if error}
        <p class="text-rose text-sm mb-3">{error}</p>
      {/if}

      <!-- Actions -->
      <div class="flex items-center gap-2.5 flex-wrap">
        {#if bet.status === 'proposed' && isOpponent()}
          <button class="inline-flex items-center justify-center gap-1.5 px-5 py-2.5 font-display text-sm font-semibold bg-lime text-white rounded-md hover:bg-lime-hover active:scale-95 transition-all duration-150 cursor-pointer border-none" onclick={doAccept} disabled={acting}>
            <Check size={16} />
            Accept
          </button>
          <button class="inline-flex items-center justify-center gap-1.5 px-5 py-2.5 font-display text-sm font-semibold bg-transparent text-text-2 border border-border rounded-md hover:bg-raised hover:text-text-1 hover:border-border-strong active:scale-95 transition-all duration-150 cursor-pointer" onclick={doDecline} disabled={acting}>
            <X size={16} />
            Decline
          </button>
        {/if}
        {#if bet.status === 'proposed' && isCreator()}
          <button class="inline-flex items-center justify-center gap-1.5 px-5 py-2.5 font-display text-sm font-semibold bg-transparent text-text-2 border border-border rounded-md hover:bg-raised hover:text-text-1 hover:border-border-strong active:scale-95 transition-all duration-150 cursor-pointer" onclick={doCancel} disabled={acting}>
            <Ban size={16} />
            Cancel
          </button>
        {/if}
        {#if bet.status === 'open' && isCreator()}
          <button class="inline-flex items-center justify-center gap-1.5 px-5 py-2.5 font-display text-sm font-semibold bg-transparent text-text-2 border border-border rounded-md hover:bg-raised hover:text-text-1 hover:border-border-strong active:scale-95 transition-all duration-150 cursor-pointer" onclick={doCancel} disabled={acting}>
            <Ban size={16} />
            Cancel
          </button>
        {/if}
        {#if canTakeOpenBet()}
          <button class="inline-flex items-center justify-center gap-1.5 px-5 py-2.5 font-display text-sm font-semibold bg-lime text-white rounded-md hover:bg-lime-hover active:scale-95 transition-all duration-150 cursor-pointer border-none" onclick={doTakeOpen} disabled={acting || $coinBalance < bet.amount}>
            <Swords size={16} />
            {#if $coinBalance < bet.amount}
              Need {formatCoins(bet.amount)} coins
            {:else}
              Take Bet
            {/if}
          </button>
        {/if}
        {#if bet.status === 'active' && (isCreator() || isOpponent())}
          <span class="text-sm text-text-3 font-semibold">Need help resolving?</span>
          <button class="inline-flex items-center justify-center gap-1.5 px-5 py-2.5 font-display text-sm font-semibold bg-lime text-white rounded-md hover:bg-lime-hover active:scale-95 transition-all duration-150 cursor-pointer border-none" onclick={doSettle} disabled={acting}>
            <Trophy size={16} />
            Mark Disputed
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>
