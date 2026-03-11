<script lang="ts">
  import { onMount } from 'svelte';
  import { coinBalance } from '$lib/stores';
  import { runWhenAuthResolved } from '$lib/auth';
  import { getMe, getBalance, claimDailyBonus, getTransactions } from '$lib/api';
  import { Gift, Coins, TrendingUp, Trophy, Target } from 'lucide-svelte';

  let profile = $state<any>(null);
  let wallet = $state<any>(null);
  let transactions = $state<any[]>([]);
  let loading = $state(true);
  let claiming = $state(false);

  onMount(() => {
    return runWhenAuthResolved(async () => {
      loading = true;
      try {
        const [me, w, txns] = await Promise.all([getMe(), getBalance(), getTransactions(20)]);
        profile = me;
        wallet = w;
        transactions = txns;
      } catch {
        profile = null;
        wallet = null;
        transactions = [];
      } finally {
        loading = false;
      }
    }, async () => {
      profile = null;
      wallet = null;
      transactions = [];
      loading = false;
    });
  });

  function formatCoins(n: number) { return n?.toLocaleString() ?? '0'; }

  function winRate() {
    if (!profile) return 0;
    const total = profile.wins + profile.losses;
    return total > 0 ? Math.round((profile.wins / total) * 100) : 0;
  }

  async function doClaim() {
    claiming = true;
    try {
      const res = await claimDailyBonus();
      coinBalance.set(res.new_balance);
      wallet.coin_balance = res.new_balance;
      wallet.bonus_available = false;
    } catch { /* already claimed */ }
    claiming = false;
  }

  function typeLabel(t: string) {
    const map: Record<string,string> = {
      signup_bonus: '🎁 Signup Bonus',
      daily_bonus: '🎁 Daily Bonus',
      bet_placed: '🎲 Bet Placed',
      bet_won: '🏆 Bet Won',
      bet_refund: '↩️ Refund',
    };
    return map[t] || t;
  }
</script>

<svelte:head>
  <title>Profile — SideBet</title>
</svelte:head>

<div class="max-w-2xl">
  {#if loading}
    <p class="text-text-3 text-sm py-10 text-center">Loading…</p>
  {:else if !profile}
    <p class="text-text-3 text-sm py-10 text-center">Sign in to view your profile.</p>
  {:else}
    <!-- Header -->
    <div class="flex items-center gap-5 mb-6 animate-enter">
      <span class="w-16 h-16 rounded-full flex items-center justify-center font-display font-bold text-2xl text-white bg-rose shrink-0">{profile.display_name?.[0]?.toUpperCase() || '?'}</span>
      <div>
        <h1 class="text-2xl font-display tracking-tight mb-0.5">{profile.display_name}</h1>
        <span class="text-sm text-text-3">@{profile.username}</span>
      </div>
    </div>

    <!-- Wallet Card -->
    <div class="flex items-center justify-between max-sm:flex-col max-sm:items-start max-sm:gap-4 bg-lime-dim border border-border rounded-lg p-6 mb-6 animate-enter" style="animation-delay: 80ms">
      <div>
        <span class="flex items-center gap-1.5 text-xs font-semibold uppercase text-text-3 mb-1">
          <Coins size={14} />
          Sweepstakes Coins
        </span>
        <span class="text-[1.75rem] font-extrabold text-lime-hover font-mono tabular-nums">{formatCoins(wallet?.coin_balance || 0)}</span>
      </div>
      <div>
        {#if wallet?.bonus_available}
          <button class="inline-flex items-center justify-center gap-1.5 px-5 py-2.5 font-display text-sm font-semibold bg-lime text-white rounded-md hover:bg-lime-hover active:scale-95 transition-all duration-150 cursor-pointer border-none" onclick={doClaim} disabled={claiming}>
            <Gift size={16} />
            {claiming ? 'Claiming…' : 'Claim Daily Bonus (+100)'}
          </button>
        {:else}
          <span class="text-[0.8125rem] text-text-3">Daily bonus claimed ✓</span>
        {/if}
      </div>
    </div>

    <!-- Stats -->
    <div class="grid grid-cols-4 max-sm:grid-cols-2 gap-3 mb-8 animate-enter" style="animation-delay: 160ms">
      <div class="bg-surface border border-border rounded-md p-4 text-center hover:border-border-strong transition-colors duration-200">
        <span class="block text-xl font-extrabold mb-1 text-lime"><Target size={16} class="inline -mt-0.5" /> {winRate()}%</span>
        <span class="text-[0.6875rem] font-semibold uppercase text-text-3">Win Rate</span>
      </div>
      <div class="bg-surface border border-border rounded-md p-4 text-center hover:border-border-strong transition-colors duration-200">
        <span class="block text-xl font-extrabold mb-1 text-text-1"><Trophy size={16} class="inline -mt-0.5" /> {profile.wins}</span>
        <span class="text-[0.6875rem] font-semibold uppercase text-text-3">Wins</span>
      </div>
      <div class="bg-surface border border-border rounded-md p-4 text-center hover:border-border-strong transition-colors duration-200">
        <span class="block text-xl font-extrabold mb-1 text-text-1">{profile.losses}</span>
        <span class="text-[0.6875rem] font-semibold uppercase text-text-3">Losses</span>
      </div>
      <div class="bg-surface border border-border rounded-md p-4 text-center hover:border-border-strong transition-colors duration-200">
        <span class="block text-xl font-extrabold mb-1 text-text-1 font-mono tabular-nums"><TrendingUp size={16} class="inline -mt-0.5" /> {formatCoins(profile.total_wagered)}</span>
        <span class="text-[0.6875rem] font-semibold uppercase text-text-3">Total Wagered</span>
      </div>
    </div>

    <!-- Transaction History -->
    {#if transactions.length > 0}
      <section class="animate-enter" style="animation-delay: 240ms">
        <h2 class="text-base font-display mb-3">Recent Transactions</h2>
        <div class="bg-surface border border-border rounded-md overflow-hidden">
          {#each transactions as txn, i}
            <div class="flex justify-between items-center px-4 py-3 border-b border-border last:border-b-0 text-sm {i % 2 === 1 ? 'bg-raised/20' : ''}">
              <span class="text-text-2">{typeLabel(txn.type)}</span>
              <span class="font-bold font-mono tabular-nums {txn.amount > 0 ? 'text-lime' : 'text-rose'}">
                {txn.amount > 0 ? '+' : ''}{formatCoins(txn.amount)}
              </span>
            </div>
          {/each}
        </div>
      </section>
    {/if}
  {/if}
</div>
