<script lang="ts">
  import { goto } from '$app/navigation';
  import { user, isAuthenticated } from '$lib/stores';

  let mode = $state<'login' | 'signup'>('login');
  let email = $state('');
  let password = $state('');
  let name = $state('');
  let error = $state('');
  let loading = $state(false);

  async function handleSubmit() {
    error = '';
    loading = true;
    try {
      // Placeholder — will call auth sidecar
      await new Promise(r => setTimeout(r, 600));
      user.set({ name: name || email.split('@')[0], email });
      isAuthenticated.set(true);
      goto('/');
    } catch (e: any) {
      error = e.message || 'Something went wrong';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head><title>SideBet — {mode === 'login' ? 'Sign In' : 'Sign Up'}</title></svelte:head>

<div class="auth-page">
  <div class="auth-left">
    <div class="brand-content">
      <span class="brand-mark">SIDEBET</span>
      <h1 class="brand-tagline">Bet your friends.<br/>Settle on-chain.</h1>
      <p class="brand-sub">Peer-to-peer wagers on NBA, politics, and anything else — trustlessly settled with UMA's oracle.</p>
    </div>
    <div class="brand-glow"></div>
  </div>

  <div class="auth-right">
    <div class="auth-form-wrap">
      <h2>{mode === 'login' ? 'Welcome back' : 'Create account'}</h2>

      {#if error}
        <div class="error-msg">{error}</div>
      {/if}

      <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
        {#if mode === 'signup'}
          <div class="field">
            <label class="label" for="name">Name</label>
            <input id="name" class="input" type="text" bind:value={name} placeholder="Your name" required />
          </div>
        {/if}

        <div class="field">
          <label class="label" for="email">Email</label>
          <input id="email" class="input" type="email" bind:value={email} placeholder="you@example.com" required />
        </div>

        <div class="field">
          <label class="label" for="pw">Password</label>
          <input id="pw" class="input" type="password" bind:value={password} placeholder="••••••••" required minlength="8" />
        </div>

        <button class="btn btn-primary btn-lg full-w" type="submit" disabled={loading}>
          {loading ? 'Loading…' : mode === 'login' ? 'Sign In' : 'Create Account'}
        </button>
      </form>

      <div class="toggle-mode">
        {#if mode === 'login'}
          Don't have an account? <button class="link-btn" onclick={() => mode = 'signup'}>Sign up</button>
        {:else}
          Already have an account? <button class="link-btn" onclick={() => mode = 'login'}>Sign in</button>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .auth-page {
    display: flex;
    min-height: calc(100vh - var(--nav-height) - 72px);
    margin: -36px -24px;
  }

  /* ── Left: Brand ── */
  .auth-left {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
    padding: 48px;
  }

  .brand-content { position: relative; z-index: 1; max-width: 400px; }
  .brand-mark {
    font-family: var(--font-display);
    font-size: 0.875rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--lime);
    margin-bottom: 20px;
    display: block;
  }
  .brand-tagline {
    font-size: 2.5rem;
    font-weight: 700;
    line-height: 1.1;
    margin-bottom: 16px;
  }
  .brand-sub {
    font-size: 0.9375rem;
    color: var(--text-2);
    line-height: 1.5;
  }

  .brand-glow {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 500px;
    height: 500px;
    background: radial-gradient(ellipse, rgba(200,255,0,0.04) 0%, transparent 70%);
    pointer-events: none;
  }

  /* ── Right: Form ── */
  .auth-right {
    width: 420px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 48px;
    border-left: 1px solid var(--border);
  }

  .auth-form-wrap { width: 100%; max-width: 320px; }
  .auth-form-wrap h2 { margin-bottom: 28px; }

  .field { margin-bottom: 18px; }
  .full-w { width: 100%; }

  .error-msg {
    background: var(--rose-dim);
    color: var(--rose);
    padding: 8px 12px;
    border-radius: var(--r-md);
    font-size: 0.8125rem;
    margin-bottom: 16px;
  }

  .toggle-mode {
    margin-top: 20px;
    font-size: 0.8125rem;
    color: var(--text-3);
    text-align: center;
  }

  .link-btn {
    background: none;
    border: none;
    color: var(--lime);
    font-family: var(--font-display);
    font-weight: 600;
    cursor: pointer;
    font-size: 0.8125rem;
  }
  .link-btn:hover { color: var(--lime-hover); }

  @media (max-width: 768px) {
    .auth-page { flex-direction: column; }
    .auth-left { padding: 32px 24px; }
    .auth-right { width: 100%; border-left: none; border-top: 1px solid var(--border); padding: 32px 24px; }
    .brand-tagline { font-size: 1.75rem; }
  }
</style>
