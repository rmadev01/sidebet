<script lang="ts">
  import { onMount } from 'svelte';
  import { coinBalance, isAuthenticated } from '$lib/stores';
  import { runWhenAuthResolved } from '$lib/auth';
  import { getOpenBets, takeBet } from '$lib/api';
  import { goto } from '$app/navigation';
  import { Plus, Clock, Swords } from 'lucide-svelte';

  let bets = $state<any[]>([]);
  let loading = $state(true);
  let taking = $state<string | null>(null);
  let error = $state('');

  onMount(() => {
    return runWhenAuthResolved(load, async () => {
      bets = [];
      loading = false;
      error = '';
    });
  });

  async function load() {
    loading = true;
    error = '';
    try {
      bets = await getOpenBets();
    } catch (e: any) {
      error = e.message;
    }
    loading = false;
  }

  async function handleTake(betId: string, amount: number) {
    taking = betId;
    error = '';
    try {
      await takeBet(betId);
      coinBalance.update(b => b - amount);
      goto(`/bets/${betId}`);
    } catch (e: any) {
      error = e.message;
      taking = null;
    }
  }

  function formatCoins(n: number) { return n?.toLocaleString() ?? '0'; }

  function timeLeft(expiresAt: string): string {
    const diff = new Date(expiresAt).getTime() - Date.now();
    if (diff <= 0) return 'Expired';
    const hours = Math.floor(diff / 3_600_000);
    const mins = Math.floor((diff % 3_600_000) / 60_000);
    if (hours > 24) return `${Math.floor(hours / 24)}d left`;
    if (hours > 0) return `${hours}h ${mins}m left`;
    return `${mins}m left`;
  }
</script>

<svelte:head>
  <title>Feed — SideBet</title>
  <meta name="description" content="Browse open bets from all users and take the ones you like." />
</svelte:head>

<div class="max-w-5xl">
  <div class="flex items-start justify-between mb-6 gap-4 flex-wrap">
    <div>
      <h1 class="text-2xl font-display tracking-tight mb-1">Open Bets</h1>
      <p class="text-text-3 text-sm">Browse bets from all users — take any bet to match it.</p>
    </div>
    <a href="/bets/new" class="inline-flex items-center justify-center gap-1.5 px-4 py-2.5 font-display text-[0.8125rem] font-semibold bg-lime text-white rounded-md no-underline hover:bg-lime-hover active:scale-95 transition-all duration-150">
      <Plus size={14} strokeWidth={2.5} />
      Post a Bet
    </a>
  </div>

  {#if !$isAuthenticated}
    <div class="text-center py-10">
      <p class="text-text-3 mb-4">🔒 Sign in to browse and take open bets.</p>
      <a href="/login" class="inline-flex items-center justify-center px-5 py-2.5 font-display text-sm font-semibold bg-lime text-white rounded-md no-underline hover:bg-lime-hover active:scale-95 transition-all duration-150">Sign In</a>
    </div>
  {:else if loading}
    <div class="text-text-3 text-sm py-10 text-center">Loading open bets…</div>
  {:else if error}
    <div class="text-rose text-sm py-10 text-center">{error}</div>
  {:else if bets.length === 0}
    <div class="text-center py-10">
      <p class="text-text-3 mb-2">🌐 No open bets right now.</p>
      <p class="text-text-3 text-sm">Be the first — <a href="/bets/new" class="text-lime underline">post a bet</a> for everyone to see.</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each bets as bet (bet.id)}
        <div class="bg-surface border border-border rounded-lg p-6 flex flex-col gap-4 transition-all duration-200 hover:border-border-strong hover:shadow-sm animate-enter">
          <!-- Header -->
          <div class="flex justify-between items-center">
            <span class="font-semibold text-sm text-lime">@{bet.creator_username}</span>
            <span class="flex items-center gap-1 text-xs text-text-3 font-medium">
              <Clock size={12} />
              {timeLeft(bet.expires_at)}
            </span>
          </div>

          <!-- Question -->
          <h3 class="text-lg font-bold leading-snug text-text-1 font-display tracking-tight">{bet.question}</h3>

          <!-- Positions -->
          <div class="flex items-center gap-3 max-sm:flex-col">
            <div class="flex-1 bg-raised rounded-md p-3 text-center max-sm:w-full">
              <span class="block text-[0.6875rem] font-semibold uppercase text-text-3 mb-1">Creator's pick</span>
              <span class="font-bold text-sm text-text-1">{bet.creator_position}</span>
            </div>
            <span class="text-xs text-text-3 font-bold shrink-0 flex items-center"><Swords size={14} /></span>
            <div class="flex-1 bg-raised rounded-md p-3 text-center max-sm:w-full">
              <span class="block text-[0.6875rem] font-semibold uppercase text-text-3 mb-1">You'd pick</span>
              <span class="font-bold text-sm text-text-1">{bet.opponent_position}</span>
            </div>
          </div>

          <!-- Meta -->
          <div class="flex gap-4">
            <div class="flex-1 bg-raised rounded-sm p-2.5 text-center">
              <span class="block text-[0.6875rem] font-semibold uppercase text-text-3 mb-0.5">Wager</span>
              <span class="font-bold text-[0.9375rem] text-text-1 font-mono tabular-nums">{formatCoins(bet.amount)}</span>
            </div>
            <div class="flex-1 bg-raised rounded-sm p-2.5 text-center">
              <span class="block text-[0.6875rem] font-semibold uppercase text-text-3 mb-0.5">Odds</span>
              <span class="font-bold text-[0.9375rem] text-text-1 font-mono tabular-nums">{bet.odds_numerator}:{bet.odds_denominator}</span>
            </div>
          </div>

          <!-- Take Button -->
          <button
            class="w-full mt-auto inline-flex items-center justify-center px-5 py-3 font-display text-sm font-semibold rounded-md transition-all duration-150 cursor-pointer border-none min-h-11
              {taking === bet.id || $coinBalance < bet.amount
                ? 'bg-raised text-text-3 cursor-not-allowed'
                : 'bg-lime text-white hover:bg-lime-hover active:scale-[0.97]'}"
            onclick={() => handleTake(bet.id, bet.amount)}
            disabled={taking === bet.id || $coinBalance < bet.amount}
          >
            {#if taking === bet.id}
              Taking…
            {:else if $coinBalance < bet.amount}
              Not enough coins
            {:else}
              Take This Bet · {formatCoins(bet.amount)} coins
            {/if}
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>
