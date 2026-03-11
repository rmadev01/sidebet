<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { authReady, coinBalance, isAuthenticated, user } from '$lib/stores';
  import { getBalance, getSession, signOut } from '$lib/api';
  import { page } from '$app/stores';
  import { LayoutDashboard, Zap, Rss, Hexagon, Users, UserCircle, Coins, Plus, Menu, X } from 'lucide-svelte';

  let { children } = $props();
  let mobileOpen = $state(false);

  const nav = [
    { href: '/', label: 'Dashboard', icon: LayoutDashboard },
    { href: '/events', label: 'Events', icon: Zap },
    { href: '/feed', label: 'Feed', icon: Rss },
    { href: '/bets', label: 'Bets', icon: Hexagon },
    { href: '/friends', label: 'Friends', icon: Users },
    { href: '/profile', label: 'Profile', icon: UserCircle }
  ];

  onMount(async () => {
    try {
      const session = await getSession();
      if (session?.user) {
        user.set(session.user);
        isAuthenticated.set(true);
        try {
          const wallet = await getBalance();
          coinBalance.set(wallet.coin_balance);
        } catch {
          coinBalance.set(0);
        }
      } else {
        user.set(null);
        isAuthenticated.set(false);
        coinBalance.set(0);
      }
    } catch {
      user.set(null);
      isAuthenticated.set(false);
      coinBalance.set(0);
    } finally {
      authReady.set(true);
    }
  });

  function isActive(href: string): boolean {
    if (href === '/') return $page.url.pathname === '/';
    return $page.url.pathname.startsWith(href);
  }

  function formatCoins(n: number): string {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
    return n.toLocaleString();
  }

  async function handleSignOut() {
    await signOut();
    user.set(null);
    isAuthenticated.set(false);
    coinBalance.set(0);
    authReady.set(true);
    mobileOpen = false;
    goto('/login');
  }
</script>

<div class="min-h-screen flex flex-col">
  <header class="sticky top-0 z-50 bg-surface border-b border-border">
    <div class="max-w-[1200px] mx-auto px-6 h-14 flex items-center justify-between gap-8">
      <a href="/" class="font-display text-[1.0625rem] font-bold tracking-[0.08em] text-lime no-underline shrink-0 hover:text-lime-hover transition-colors duration-150">
        SIDEBET
      </a>

      <nav class="hidden md:flex items-center gap-1">
        {#each nav as item}
          <a
            href={item.href}
            class="flex items-center gap-1.5 text-[0.8125rem] font-medium no-underline px-3.5 py-1.5 rounded-md transition-all duration-150
              {isActive(item.href)
                ? 'text-text-1 bg-raised'
                : 'text-text-3 hover:text-text-2 hover:bg-raised'}"
            onclick={() => mobileOpen = false}
          >
            <item.icon size={15} strokeWidth={2} />
            {item.label}
          </a>
        {/each}
      </nav>

      <div class="flex items-center gap-2.5 shrink-0">
        {#if $isAuthenticated}
          <span class="flex items-center gap-1.5 bg-lime-dim px-2.5 py-1 rounded-full font-mono text-xs font-bold text-lime-hover cursor-default" title="Your coin balance">
            <Coins size={14} strokeWidth={2.5} />
            <span class="leading-none">{formatCoins($coinBalance)}</span>
          </span>
          <a href="/bets/new" class="inline-flex items-center justify-center gap-1.5 px-3 py-1.5 font-display text-xs font-semibold bg-lime text-white rounded-md no-underline hover:bg-lime-hover active:scale-95 transition-all duration-150 whitespace-nowrap">
            <Plus size={14} strokeWidth={2.5} />
            New Bet
          </a>
          <a href="/profile" aria-label="Open profile" class="w-7 h-7 rounded-full flex items-center justify-center font-display font-bold text-[0.625rem] text-white bg-lime shrink-0 no-underline">
            {$user?.name?.[0]?.toUpperCase() || '?'}
          </a>
          <button class="hidden md:inline-flex items-center justify-center px-3 py-1.5 font-display text-xs font-semibold bg-transparent text-text-2 border border-border rounded-md hover:bg-raised hover:text-text-1 active:scale-95 transition-all duration-150 cursor-pointer" onclick={handleSignOut}>
            Sign Out
          </button>
        {:else}
          <a href="/login" class="inline-flex items-center justify-center px-3 py-1.5 font-display text-xs font-semibold bg-lime text-white rounded-md no-underline hover:bg-lime-hover active:scale-95 transition-all duration-150">Sign In</a>
        {/if}
        <button class="flex md:hidden w-[34px] h-[34px] items-center justify-center bg-transparent border border-border rounded-md text-text-2 cursor-pointer" onclick={() => mobileOpen = !mobileOpen} aria-label="Menu">
          {#if mobileOpen}
            <X size={18} />
          {:else}
            <Menu size={18} />
          {/if}
        </button>
      </div>
    </div>

    {#if mobileOpen}
      <nav class="md:hidden flex flex-col gap-0.5 px-4 py-3 border-t border-border bg-surface animate-enter">
        {#each nav as item}
          <a
            href={item.href}
            class="flex items-center gap-2 text-[0.8125rem] font-medium no-underline w-full px-3.5 py-2.5 rounded-md transition-all duration-150
              {isActive(item.href)
                ? 'text-text-1 bg-raised'
                : 'text-text-3 hover:text-text-2 hover:bg-raised'}"
            onclick={() => mobileOpen = false}
          >
            <item.icon size={16} strokeWidth={2} />
            {item.label}
          </a>
        {/each}
        {#if $isAuthenticated}
          <button class="flex items-center gap-2 text-[0.8125rem] font-medium w-full px-3.5 py-2.5 rounded-md transition-all duration-150 cursor-pointer border border-border bg-transparent text-text-3 hover:text-text-2 hover:bg-raised" onclick={handleSignOut}>
            Sign Out
          </button>
        {/if}
      </nav>
    {/if}
  </header>

  <main class="flex-1 py-9 pb-16 max-md:py-6 max-md:pb-12">
    <div class="max-w-[1200px] mx-auto px-6 max-sm:px-4">
      {@render children()}
    </div>
  </main>
</div>
