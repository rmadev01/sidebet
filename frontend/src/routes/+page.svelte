<script lang="ts">
  import { onMount } from 'svelte';
  import { runWhenAuthResolved } from '$lib/auth';
  import { getFeed, getBets, getMe } from '$lib/api';
  import { TrendingUp, Trophy, Coins, ArrowRight, CalendarClock, ShieldCheck, Users } from 'lucide-svelte';

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

<div class="max-w-6xl">
  <section class="grid lg:grid-cols-[minmax(0,1.15fr)_360px] gap-6 items-stretch py-8 pb-7 animate-enter">
    <div class="panel rounded-[32px] p-7 sm:p-8 overflow-hidden relative">
      <div class="absolute inset-x-0 top-0 h-28 bg-[linear-gradient(135deg,rgba(31,122,104,0.16),rgba(248,195,106,0.16))]"></div>
      <div class="relative">
        <div class="section-kicker mb-4">Command Center</div>
        {#if stats}
          <h1 class="text-4xl max-sm:text-[1.9rem] font-bold font-display tracking-tight text-balance max-w-2xl">
            You're running <span class="text-lime">{stats.wins} wins</span> against {stats.losses} losses this month.
          </h1>
          <p class="mt-4 text-text-2 max-w-xl text-balance">
            Keep the pressure on. Your wallet is loaded, the feed is live, and there are fresh challenges waiting for an answer.
          </p>
        {:else}
          <h1 class="text-4xl max-sm:text-[1.9rem] font-bold font-display tracking-tight text-balance max-w-2xl">
            Build rivalries around <span class="text-lime">sports, events, and bragging rights</span>.
          </h1>
          <p class="mt-4 text-text-2 max-w-xl text-balance">
            SideBet gives every new account a sweepstakes-style coin stack so you can challenge friends without touching real money.
          </p>
        {/if}

        <div class="grid sm:grid-cols-3 gap-3 mt-7">
          <div class="panel-soft rounded-2xl p-4">
            <div class="flex items-center gap-2 text-sm text-text-2 mb-2"><Coins size={16} class="text-lime" /> Wallet</div>
            <div class="font-display text-2xl">{formatCoins(stats?.coin_balance ?? 10000)}</div>
            <div class="text-xs text-text-3 mt-1">Coins ready for your next callout.</div>
          </div>
          <div class="panel-soft rounded-2xl p-4">
            <div class="flex items-center gap-2 text-sm text-text-2 mb-2"><TrendingUp size={16} class="text-lime" /> Pending</div>
            <div class="font-display text-2xl">{pendingBets.length}</div>
            <div class="text-xs text-text-3 mt-1">Challenges currently waiting on you.</div>
          </div>
          <div class="panel-soft rounded-2xl p-4">
            <div class="flex items-center gap-2 text-sm text-text-2 mb-2"><Trophy size={16} class="text-lime" /> Activity</div>
            <div class="font-display text-2xl">{feedItems.length}</div>
            <div class="text-xs text-text-3 mt-1">Recent feed items across your circle.</div>
          </div>
        </div>

        <div class="flex flex-wrap items-center gap-3 mt-7">
          <a href="/bets/new" class="inline-flex items-center gap-2 px-4 py-2.5 rounded-2xl bg-lime text-white font-display text-sm font-semibold no-underline shadow-[0_12px_28px_rgba(31,122,104,0.18)] hover:bg-lime-hover transition-colors">
            Start a Bet
            <ArrowRight size={16} />
          </a>
          <a href="/events" class="inline-flex items-center gap-2 px-4 py-2.5 rounded-2xl border border-border text-text-1 font-display text-sm font-semibold no-underline hover:bg-raised transition-colors">
            Browse Events
            <CalendarClock size={16} />
          </a>
        </div>
      </div>
    </div>

    <div class="panel rounded-[32px] p-5 sm:p-6 flex flex-col gap-5 animate-enter" style="animation-delay: 70ms">
      <img src="/sidebet-hero.svg" alt="SideBet dashboard illustration" class="w-full rounded-[24px] border border-white/30 bg-[#173230]" />
      <div class="grid gap-3">
        <div class="panel-soft rounded-2xl p-4 flex items-start gap-3">
          <div class="w-10 h-10 rounded-2xl bg-lime-dim text-lime flex items-center justify-center shrink-0"><ShieldCheck size={18} /></div>
          <div>
            <div class="font-display text-sm">Virtual-coin only</div>
            <div class="text-xs text-text-3 mt-1">Designed for stakes, receipts, and social rivalry without real-money risk.</div>
          </div>
        </div>
        <div class="panel-soft rounded-2xl p-4 flex items-start gap-3">
          <div class="w-10 h-10 rounded-2xl bg-amber-dim text-amber flex items-center justify-center shrink-0"><Users size={18} /></div>
          <div>
            <div class="font-display text-sm">Built for friend groups</div>
            <div class="text-xs text-text-3 mt-1">Post a public challenge or target a specific friend when it needs to get personal.</div>
          </div>
        </div>
      </div>
    </div>
  </section>

  {#if pendingBets.length > 0}
    <section class="mb-6 animate-enter" style="animation-delay: 90ms">
      <div class="flex items-center justify-between mb-4">
        <h2 class="flex items-center gap-2 text-base font-display">
          <span class="w-1.5 h-1.5 rounded-full bg-lime animate-pulse-live inline-block"></span>
          Incoming challenges
        </h2>
        <span class="font-mono text-xs text-text-3 bg-raised px-2 py-0.5 rounded-full">{pendingBets.length}</span>
      </div>

      <div class="panel rounded-[28px] p-2 stagger">
        {#each pendingBets as bet}
          <a href="/bets/{bet.id}" class="group flex items-center max-sm:flex-col max-sm:items-start justify-between gap-4 max-sm:gap-2 py-4 px-4 sm:px-5 border-b border-border no-underline text-inherit hover:bg-raised/45 transition-colors duration-150 last:border-b-0 rounded-2xl relative before:content-[''] before:absolute before:left-1 before:top-2 before:bottom-2 before:w-[3px] before:rounded-sm before:bg-amber">
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

  <section class="mb-10 animate-enter" style="animation-delay: 140ms">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-base font-display">Recent activity</h2>
    </div>

    {#if feedItems.length === 0 && !loading}
      <div class="panel rounded-[28px] p-6 text-sm text-text-3">
        No activity yet. Add friends and place bets to light up the feed.
      </div>
    {:else}
      <div class="panel rounded-[28px] p-2 stagger">
        {#each feedItems as item}
          <div class="flex items-center max-sm:flex-col max-sm:items-start justify-between gap-4 max-sm:gap-2 py-3.5 px-4 sm:px-5 border-b border-border last:border-b-0 rounded-2xl relative before:content-[''] before:absolute before:left-1 before:top-2 before:bottom-2 before:w-[3px] before:rounded-sm
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
    {/if}
  </section>
</div>
