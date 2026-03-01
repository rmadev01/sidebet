<script lang="ts">
  import { goto } from '$app/navigation';
  import { user, isAuthenticated } from '$lib/stores';
  import { signIn, signUp } from '$lib/api';

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
      if (mode === 'signup') {
        const res = await signUp(email, password, name);
        if (res.error) throw new Error(res.error.message || 'Signup failed');
      }
      const res = await signIn(email, password);
      if (res.error) throw new Error(res.error.message || 'Login failed');
      user.set(res.user);
      isAuthenticated.set(true);
      goto('/');
    } catch (e: any) {
      error = e.message || 'Something went wrong';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>SideBet — {mode === 'login' ? 'Sign In' : 'Sign Up'}</title>
</svelte:head>

<div class="login-page">
  <div class="login-card animate-slide-up">
    <div class="login-header">
      <span style="font-size:2.5rem">🎯</span>
      <h1>SideBet</h1>
      <p class="text-secondary">
        {mode === 'login' ? 'Sign in to your account' : 'Create your account'}
      </p>
    </div>

    {#if error}
      <div class="error-msg">{error}</div>
    {/if}

    <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      {#if mode === 'signup'}
        <div class="form-group">
          <label class="label" for="name">Display Name</label>
          <input id="name" class="input" bind:value={name} placeholder="Your name" required />
        </div>
      {/if}
      <div class="form-group">
        <label class="label" for="email">Email</label>
        <input id="email" class="input" type="email" bind:value={email} placeholder="you@example.com" required />
      </div>
      <div class="form-group">
        <label class="label" for="password">Password</label>
        <input id="password" class="input" type="password" bind:value={password} placeholder="••••••••" required minlength="8" />
      </div>
      <button class="btn btn-primary btn-lg" style="width:100%" type="submit" disabled={loading}>
        {loading ? '...' : mode === 'login' ? 'Sign In' : 'Create Account'}
      </button>
    </form>

    <p class="toggle-text text-sm text-muted">
      {#if mode === 'login'}
        Don't have an account? <button class="link-btn" onclick={() => mode = 'signup'}>Sign up</button>
      {:else}
        Already have an account? <button class="link-btn" onclick={() => mode = 'login'}>Sign in</button>
      {/if}
    </p>
  </div>
</div>

<style>
  .login-page {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 80vh;
  }
  .login-card {
    width: 100%;
    max-width: 420px;
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
    padding: var(--space-2xl);
  }
  .login-header {
    text-align: center;
    margin-bottom: var(--space-2xl);
  }
  .login-header h1 {
    background: linear-gradient(135deg, var(--accent-blue), var(--accent-purple));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    font-weight: 800;
  }
  .form-group { margin-bottom: var(--space-lg); }
  .error-msg {
    background: var(--accent-red-dim);
    color: var(--accent-red);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-sm);
    font-size: 0.875rem;
    margin-bottom: var(--space-lg);
  }
  .toggle-text { text-align: center; margin-top: var(--space-lg); }
  .link-btn {
    background: none;
    border: none;
    color: var(--accent-blue);
    cursor: pointer;
    font-family: var(--font-sans);
    font-size: inherit;
    font-weight: 600;
    text-decoration: underline;
  }
</style>
