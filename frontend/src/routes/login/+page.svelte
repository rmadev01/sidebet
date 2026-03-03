<script lang="ts">
  import { goto } from '$app/navigation';
  import { user, isAuthenticated, coinBalance } from '$lib/stores';
  import { signIn, signUp, getSession, getBalance } from '$lib/api';

  let mode = $state<'signin' | 'signup'>('signin');
  let email = $state('');
  let password = $state('');
  let name = $state('');
  let error = $state('');
  let loading = $state(false);

  async function handleSubmit() {
    error = '';
    loading = true;

    try {
      if (mode === 'signup') {
        if (!name.trim()) { error = 'Name is required'; loading = false; return; }
        const res = await signUp(email, password, name);
        if (res?.error) { error = res.error.message || 'Signup failed'; loading = false; return; }
      } else {
        const res = await signIn(email, password);
        if (res?.error) { error = res.error.message || 'Invalid credentials'; loading = false; return; }
      }

      // Fetch session
      const session = await getSession();
      if (session?.user) {
        user.set(session.user);
        isAuthenticated.set(true);
        try {
          const wallet = await getBalance();
          coinBalance.set(wallet.coin_balance);
        } catch { /* ok */ }
        goto('/');
      } else {
        error = 'Authentication failed. Please try again.';
      }
    } catch (e: any) {
      error = e.message || 'Something went wrong';
    }
    loading = false;
  }
</script>

<svelte:head>
  <title>Sign In — SideBet</title>
</svelte:head>

<div class="login-page">
  <div class="login-card animate-in">
    <div class="login-logo">SIDEBET</div>
    <h1>{mode === 'signin' ? 'Welcome back' : 'Create account'}</h1>
    <p class="sub">Bet on events with friends. Play with coins, not cash.</p>

    <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      {#if mode === 'signup'}
        <div class="form-group">
          <label for="name">Name</label>
          <input id="name" type="text" class="input" bind:value={name} placeholder="Your name" required />
        </div>
      {/if}

      <div class="form-group">
        <label for="email">Email</label>
        <input id="email" type="email" class="input" bind:value={email} placeholder="you@example.com" required />
      </div>

      <div class="form-group">
        <label for="password">Password</label>
        <input id="password" type="password" class="input" bind:value={password} placeholder="••••••••" required minlength="8" />
      </div>

      {#if error}
        <p class="error">{error}</p>
      {/if}

      <button type="submit" class="btn btn-primary btn-full" disabled={loading}>
        {loading ? 'Working…' : mode === 'signin' ? 'Sign In' : 'Sign Up'}
      </button>
    </form>

    <p class="toggle">
      {mode === 'signin' ? "Don't have an account?" : 'Already have an account?'}
      <button class="link-btn" onclick={() => { mode = mode === 'signin' ? 'signup' : 'signin'; error = ''; }}>
        {mode === 'signin' ? 'Sign Up' : 'Sign In'}
      </button>
    </p>
  </div>
</div>

<style>
  .login-page {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: calc(100vh - var(--nav-height) - 100px);
  }

  .login-card {
    width: 100%;
    max-width: 400px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    padding: 40px 32px;
    text-align: center;
  }

  .login-logo {
    font-family: var(--font-display);
    font-size: 0.875rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--lime);
    margin-bottom: 20px;
  }

  h1 {
    font-size: 1.375rem;
    margin-bottom: 6px;
  }

  .sub {
    font-size: 0.8125rem;
    color: var(--text-3);
    margin-bottom: 28px;
  }

  .form-group {
    margin-bottom: 16px;
    text-align: left;
  }
  .form-group label {
    display: block;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-3);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-bottom: 6px;
  }

  .btn-full {
    width: 100%;
    margin-top: 8px;
  }

  .error {
    color: var(--rose);
    font-size: 0.8125rem;
    margin-bottom: 12px;
    text-align: center;
  }

  .toggle {
    margin-top: 20px;
    font-size: 0.8125rem;
    color: var(--text-3);
  }
  .link-btn {
    background: none;
    border: none;
    color: var(--lime);
    cursor: pointer;
    font-weight: 600;
    font-size: 0.8125rem;
    text-decoration: underline;
    padding: 0;
  }
  .link-btn:hover { color: var(--lime-hover); }
</style>
