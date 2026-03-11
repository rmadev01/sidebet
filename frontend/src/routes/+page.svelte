<script lang="ts">
  import { onMount } from 'svelte';
  import { runWhenAuthResolved } from '$lib/auth';
  import { getFeed, getBets, getMe } from '$lib/api';
  import { TrendingUp, Trophy, Coins } from 'lucide-svelte';

  let feedItems = $state<any[]>([]);
  let pendingBets = $state<any[]>([]);
  let stats = $state<any>(null);
  let tab = $state('active');
  let loading = $state(true);

  onMount(() => {
    return runWhenAuthResolved(async () => {
      loading = true;
      try {
        const [feed, pending, me] = await Promise.all([
          getFeed().catch(() => []),
          getBets({ status: 'proposed', role: 'opponent' }).catch(() => []),
          getMe().catch(() => null)
        ]);
        feedItems = feed;
        pendingBets = pending;
        stats = me;
      } catch {
        feedItems = [];
        pendingBets = [];
        stats = null;
      } finally {
        loading = false;
      }
    }, async () => {
      feedItems = [];
      pendingBets = [];
      stats = null;
      loading = false;
    });
  });

  function formatCoins(n: number): string {
    return n?.toLocaleString() ?? '0';
  }

  function timeAgo(dateStr: string): string {
    const diff = Date.now() - new Date(dateStr).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    return `${Math.floor(hrs / 24)}d ago`;
  }
</script>

<svelte:head>
  <title>SideBet — Dashboard</title>
  <meta name="description" content="Your peer-to-peer betting dashboard." />
</svelte:head>

<div class="max-w-5xl">
  <!-- Hero -->
  <section class="py-12 pb-10 mb-3 animate-enter">
    {#if stats}
      <h1 class="text-4xl max-sm:text-[1.625rem] font-bold font-display tracking-tight">
        You're <span class="text-lime">{stats.wins}–{stats.losses}</span> this month
      </h1>
      <div class="flex items-center gap-2.5 mt-3 text-sm text-text-2 flex-wrap">
        <span class="flex items-center gap-1"><Coins size={14} /> <strong class="font-mono tabular-nums tracking-tight">{formatCoins(stats.coin_balance)}</strong> coins</span>
        <span class="text-text-3">·</span>
        <span class="flex items-center gap-1"><TrendingUp size={14} /> <strong class="font-mono tabular-nums tracking-tight">{formatCoins(stats.total_wagered)}</strong> wagered</span>
        <span class="text-text-3">·</span>
        <span class="flex items-center gap-1 bg-amber-dim text-amber px-2.5 py-0.5 rounded-full text-xs font-semibold font-display">
          <Trophy size={12} />
          {stats.wins > stats.losses ? `${stats.wins - stats.losses}W` : '—'}
        </span>
      </div>
    {:else}
      <h1 class="text-4xl max-sm:text-[1.625rem] font-bold font-display tracking-tight">
        Welcome to <span class="text-lime">SideBet</span>
      </h1>
      <div class="flex items-center gap-2.5 mt-3 text-sm text-text-2 flex-wrap">
        <span>Sign up free — you get 10,000 sweepstakes coins to start betting with friends</span>
      </div>
    {/if}
  </section>

  <!-- Incoming Bets -->
  {#if pendingBets.length > 0}
    <section class="mb-10 animate-enter" style="animation-delay: 80ms">
      <div class="flex items-center justify-between mb-4">
        <h2 class="flex items-center gap-2 text-base font-display">
          <span class="w-1.5 h-1.5 rounded-full bg-lime animate-pulse-live inline-block"></span>
          Incoming
        </h2>
        <span class="font-mono text-xs text-text-3 bg-raised px-2 py-0.5 rounded-full">{pendingBets.length}</span>
      </div>

      <div class="stagger">
        {#each pendingBets as bet}
          <a href="/bets/{bet.id}" class="group flex items-center max-sm:flex-col max-sm:items-start justify-between gap-4 max-sm:gap-2 py-3.5 px-3.5 pl-5 border-b border-border no-underline text-inherit hover:bg-raised/50 transition-colors duration-150 last:border-b-0 relative before:content-[''] before:absolute before:left-0 before:top-1 before:bottom-1 before:w-[3px] before:rounded-sm before:bg-amber">
            <div class="flex-1 min-w-0">
              <span class="block font-semibold text-[0.9375rem] leading-snug mb-0.5 truncate">{bet.question}</span>
              <span class="text-xs text-text-3">{bet.odds_numerator}:{bet.odds_denominator} · expires {new Date(bet.expires_at).toLocaleDateString()}</span>
            </div>
            <div class="flex items-center gap-3 shrink-0 max-sm:w-full max-sm:justify-between">
              <span class="font-mono tabular-nums text-sm font-semibold text-text-1">{formatCoins(bet.amount)} coins</span>
            </div>
          </a>
        {/each}
      </div>
    </section>
  {/if}

  <!-- Feed -->
  <section class="mb-10 animate-enter" style="animation-delay: 160ms">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-base font-display">Activity</h2>
    </div>

    {#if feedItems.length === 0 && !loading}
      <p class="text-text-3 text-sm py-5">No activity yet. Add friends and place bets to see your feed!</p>
    {/if}

    <div class="stagger">
      {#each feedItems as item}
        <div class="flex items-center max-sm:flex-col max-sm:items-start justify-between gap-4 max-sm:gap-2 py-3 px-3 pl-5 border-b border-border last:border-b-0 relative before:content-[''] before:absolute before:left-0 before:top-1 before:bottom-1 before:w-[3px] before:rounded-sm
          {item.payload?.outcome?.includes('win') ? 'before:bg-lime' : item.payload?.outcome?.includes('lose') || item.payload?.outcome?.includes('loss') ? 'before:bg-rose' : 'before:bg-sky'}">
          <div class="flex-1 min-w-0">
            <span class="block text-sm font-medium leading-snug text-text-1 mb-0.5">{item.payload?.question || 'Bet update'}</span>
            <span class="text-xs text-text-3">{item.type} · {timeAgo(item.created_at)}</span>
          </div>
          <div class="flex items-center gap-2.5 shrink-0 max-sm:w-full max-sm:justify-between">
            <span class="font-mono tabular-nums text-sm font-semibold text-text-1">{formatCoins(item.payload?.amount || 0)} coins</span>
            {#if item.payload?.status === 'settled'}
              <span class="font-display text-[0.6875rem] font-semibold px-2 py-0.5 rounded-sm uppercase tracking-wide bg-lime-dim text-lime">Settled</span>
            {:else}
              <span class="font-display text-[0.6875rem] font-semibold px-2 py-0.5 rounded-sm uppercase tracking-wide bg-sky-dim text-sky">Active</span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </section>
</div>
