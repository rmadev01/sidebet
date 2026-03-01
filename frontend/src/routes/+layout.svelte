<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { user, isAuthenticated } from '$lib/stores';
  import { getSession } from '$lib/api';
  import { page } from '$app/stores';

  let { children } = $props();
  let mobileMenuOpen = $state(false);

  const navItems = [
    { href: '/', label: 'Dashboard', icon: '⚡' },
    { href: '/events', label: 'Events', icon: '🏀' },
    { href: '/bets', label: 'My Bets', icon: '🎲' },
    { href: '/bets/new', label: 'New Bet', icon: '➕' },
    { href: '/friends', label: 'Friends', icon: '👥' },
    { href: '/profile', label: 'Profile', icon: '👤' },
  ];

  onMount(async () => {
    try {
      const session = await getSession();
      if (session?.user) {
        user.set(session.user);
        isAuthenticated.set(true);
      }
    } catch (e) {
      // not logged in
    }
  });
</script>

<div class="app-layout">
  <!-- Sidebar Navigation -->
  <aside class="sidebar" class:open={mobileMenuOpen}>
    <div class="sidebar-header">
      <a href="/" class="logo">
        <span class="logo-icon">🎯</span>
        <span class="logo-text">SideBet</span>
      </a>
    </div>

    <nav class="sidebar-nav">
      {#each navItems as item}
        <a
          href={item.href}
          class="nav-item"
          class:active={$page.url.pathname === item.href}
          onclick={() => mobileMenuOpen = false}
        >
          <span class="nav-icon">{item.icon}</span>
          <span class="nav-label">{item.label}</span>
        </a>
      {/each}
    </nav>

    <div class="sidebar-footer">
      {#if $isAuthenticated}
        <div class="user-pill">
          <div class="avatar avatar-sm">
            {$user?.name?.[0]?.toUpperCase() || '?'}
          </div>
          <span class="user-name">{$user?.name || 'User'}</span>
        </div>
      {:else}
        <a href="/login" class="btn btn-primary btn-sm" style="width:100%">Sign In</a>
      {/if}
    </div>
  </aside>

  <!-- Mobile header -->
  <header class="mobile-header">
    <button class="menu-toggle" onclick={() => mobileMenuOpen = !mobileMenuOpen}>
      {mobileMenuOpen ? '✕' : '☰'}
    </button>
    <a href="/" class="logo">
      <span class="logo-icon">🎯</span>
      <span class="logo-text">SideBet</span>
    </a>
    <div style="width:36px"></div>
  </header>

  <!-- Overlay for mobile menu -->
  {#if mobileMenuOpen}
    <div class="overlay" onclick={() => mobileMenuOpen = false}></div>
  {/if}

  <!-- Main Content -->
  <main class="main-content">
    {@render children()}
  </main>
</div>

<style>
  .app-layout {
    display: flex;
    min-height: 100vh;
  }

  /* ── Sidebar ── */
  .sidebar {
    width: var(--sidebar-width);
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    z-index: 100;
  }

  .sidebar-header {
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border-subtle);
  }

  .logo {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    color: var(--text-primary);
    text-decoration: none;
  }

  .logo-icon { font-size: 1.5rem; }

  .logo-text {
    font-size: 1.25rem;
    font-weight: 800;
    letter-spacing: -0.03em;
    background: linear-gradient(135deg, var(--accent-blue), var(--accent-purple));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .sidebar-nav {
    flex: 1;
    padding: var(--space-md);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: 0.6rem 0.875rem;
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    text-decoration: none;
    font-size: 0.9375rem;
    font-weight: 500;
    transition: all var(--transition-fast);
  }
  .nav-item:hover {
    background: var(--bg-card);
    color: var(--text-primary);
  }
  .nav-item.active {
    background: var(--accent-blue-dim);
    color: var(--accent-blue);
  }

  .nav-icon { font-size: 1.125rem; width: 1.5rem; text-align: center; }

  .sidebar-footer {
    padding: var(--space-md) var(--space-lg);
    border-top: 1px solid var(--border-subtle);
  }

  .user-pill {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .user-name {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  /* ── Main ── */
  .main-content {
    flex: 1;
    margin-left: var(--sidebar-width);
    padding: var(--space-xl) var(--space-2xl);
    min-height: 100vh;
  }

  /* ── Mobile ── */
  .mobile-header {
    display: none;
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: var(--header-height);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-subtle);
    padding: 0 var(--space-md);
    align-items: center;
    justify-content: space-between;
    z-index: 90;
  }

  .menu-toggle {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 1.25rem;
    cursor: pointer;
  }

  .overlay {
    display: none;
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.6);
    z-index: 99;
  }

  @media (max-width: 768px) {
    .sidebar {
      transform: translateX(-100%);
      transition: transform var(--transition-base);
    }
    .sidebar.open {
      transform: translateX(0);
    }
    .mobile-header {
      display: flex;
    }
    .overlay { display: block; }
    .main-content {
      margin-left: 0;
      padding: calc(var(--header-height) + var(--space-lg)) var(--space-md) var(--space-lg);
    }
  }
</style>
