<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { user, isAuthenticated } from '$lib/stores';
  import { getSession } from '$lib/api';
  import { page } from '$app/stores';

  let { children } = $props();
  let mobileOpen = $state(false);

  const nav = [
    { href: '/', label: 'Dashboard' },
    { href: '/events', label: 'Events' },
    { href: '/bets', label: 'Bets' },
    { href: '/friends', label: 'Friends' },
    { href: '/profile', label: 'Profile' },
  ];

  onMount(async () => {
    try {
      const session = await getSession();
      if (session?.user) {
        user.set(session.user);
        isAuthenticated.set(true);
      }
    } catch { /* not logged in */ }
  });

  function isActive(href: string): boolean {
    if (href === '/') return $page.url.pathname === '/';
    return $page.url.pathname.startsWith(href);
  }
</script>

<div class="app">
  <!-- Top Navigation -->
  <header class="topnav">
    <div class="topnav-inner">
      <a href="/" class="wordmark">SIDEBET</a>

      <nav class="nav-links" class:open={mobileOpen}>
        {#each nav as item}
          <a
            href={item.href}
            class="nav-link"
            class:active={isActive(item.href)}
            onclick={() => mobileOpen = false}
          >{item.label}</a>
        {/each}
      </nav>

      <div class="nav-right">
        {#if $isAuthenticated}
          <a href="/bets/new" class="btn btn-primary btn-sm">New Bet</a>
          <a href="/profile" class="av av-1 av--sm">
            {$user?.name?.[0]?.toUpperCase() || '?'}
          </a>
        {:else}
          <a href="/login" class="btn btn-primary btn-sm">Sign In</a>
        {/if}
        <button class="burger" onclick={() => mobileOpen = !mobileOpen} aria-label="Menu">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            {#if mobileOpen}
              <path d="M18 6L6 18M6 6l12 12"/>
            {:else}
              <path d="M3 12h18M3 6h18M3 18h18"/>
            {/if}
          </svg>
        </button>
      </div>
    </div>
  </header>

  <!-- Main -->
  <main class="main">
    <div class="page-w">
      {@render children()}
    </div>
  </main>
</div>

<style>
  .app {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  /* ── Top Nav ── */
  .topnav {
    position: sticky;
    top: 0;
    z-index: 100;
    background: hsla(230, 20%, 6%, 0.85);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border-bottom: 1px solid var(--border);
  }

  .topnav-inner {
    max-width: var(--max-w);
    margin: 0 auto;
    padding: 0 24px;
    height: var(--nav-height);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 32px;
  }

  .wordmark {
    font-family: var(--font-display);
    font-size: 1.0625rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--lime);
    text-decoration: none;
    flex-shrink: 0;
  }
  .wordmark:hover { color: var(--lime-hover); }

  .nav-links {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .nav-link {
    font-family: var(--font-body);
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-3);
    text-decoration: none;
    padding: 6px 14px;
    border-radius: var(--r-md);
    transition: all var(--dur-fast) var(--ease-out);
  }
  .nav-link:hover {
    color: var(--text-2);
    background: var(--bg-raised);
  }
  .nav-link.active {
    color: var(--text-1);
    background: var(--bg-hover);
  }

  .nav-right {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }

  .burger {
    display: none;
    width: 34px;
    height: 34px;
    align-items: center;
    justify-content: center;
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    color: var(--text-2);
    cursor: pointer;
  }

  /* ── Main ── */
  .main {
    flex: 1;
    padding: 36px 0 64px;
  }

  /* ── Mobile ── */
  @media (max-width: 720px) {
    .burger { display: flex; }

    .nav-links {
      display: none;
      position: absolute;
      top: var(--nav-height);
      left: 0;
      right: 0;
      background: var(--bg-surface);
      border-bottom: 1px solid var(--border);
      flex-direction: column;
      padding: 12px 16px;
      gap: 2px;
    }
    .nav-links.open { display: flex; }

    .nav-link {
      width: 100%;
      padding: 10px 14px;
    }

    .main { padding: 24px 0 48px; }
  }
</style>
