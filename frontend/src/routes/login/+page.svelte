<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { authReady, coinBalance, isAuthenticated, user } from '$lib/stores';
  import { signIn, signUp, getSession, getBalance } from '$lib/api';
  import { LogIn, UserPlus } from 'lucide-svelte';

  let mode = $state<'signin' | 'signup'>('signin');
  let email = $state('');
  let password = $state('');
  let name = $state('');
  let error = $state('');
  let loading = $state(false);

  function redirectTarget() {
    const next = page.url.searchParams.get('next');
    return next && next.startsWith('/') ? next : '/';
  }

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

      const session = await getSession();
      if (session?.user) {
        user.set(session.user);
        isAuthenticated.set(true);
        authReady.set(true);
        try {
          const wallet = await getBalance();
          coinBalance.set(wallet.coin_balance);
        } catch { /* ok */ }
        goto(redirectTarget());
      } else {
        error = 'Authentication failed. Please try again.';
        authReady.set(true);
      }
    } catch (e: any) {
      error = e.message || 'Something went wrong';
      authReady.set(true);
    }
    loading = false;
  }
</script>

<svelte:head>
  <title>Sign In — SideBet</title>
</svelte:head>

<div class="flex items-center justify-center min-h-[calc(100vh-var(--nav-height,56px)-100px)]">
  <div class="w-full max-w-[400px] relative animate-enter">
    <div class="bg-surface border border-border rounded-lg px-8 py-10 text-center shadow-sm">
      <!-- Logo -->
      <div class="font-display text-sm font-bold tracking-[0.1em] text-lime mb-5">SIDEBET</div>

      <h1 class="text-[1.375rem] font-display tracking-tight mb-1.5">{mode === 'signin' ? 'Welcome back' : 'Create account'}</h1>
      <p class="text-[0.8125rem] text-text-3 mb-7">Bet on events with friends using free sweepstakes coins — no real money required.</p>

      <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="text-left">
        {#if mode === 'signup'}
          <div class="mb-4">
            <label for="name" class="block text-xs font-semibold text-text-3 uppercase tracking-wide mb-1.5">Name</label>
            <input id="name" type="text" bind:value={name} placeholder="Your name" required
              class="w-full py-2.5 px-3.5 font-body text-[0.9375rem] bg-input text-text-1 border border-border rounded-md outline-none transition-all duration-150 focus:border-lime focus:shadow-[0_0_0_2px_var(--color-lime-dim)] placeholder:text-text-3" />
          </div>
        {/if}

        <div class="mb-4">
          <label for="email" class="block text-xs font-semibold text-text-3 uppercase tracking-wide mb-1.5">Email</label>
          <input id="email" type="email" bind:value={email} placeholder="you@example.com" required
            class="w-full py-2.5 px-3.5 font-body text-[0.9375rem] bg-input text-text-1 border border-border rounded-md outline-none transition-all duration-150 focus:border-lime focus:shadow-[0_0_0_2px_var(--color-lime-dim)] placeholder:text-text-3" />
        </div>

        <div class="mb-4">
          <label for="password" class="block text-xs font-semibold text-text-3 uppercase tracking-wide mb-1.5">Password</label>
          <input id="password" type="password" bind:value={password} placeholder="••••••••" required minlength="8"
            class="w-full py-2.5 px-3.5 font-body text-[0.9375rem] bg-input text-text-1 border border-border rounded-md outline-none transition-all duration-150 focus:border-lime focus:shadow-[0_0_0_2px_var(--color-lime-dim)] placeholder:text-text-3" />
        </div>

        {#if error}
          <p class="text-rose text-[0.8125rem] mb-3 text-center">{error}</p>
        {/if}

        <button type="submit" disabled={loading}
          class="w-full mt-2 inline-flex items-center justify-center gap-1.5 px-5 py-3 font-display text-sm font-semibold bg-lime text-white rounded-md hover:bg-lime-hover active:scale-[0.97] transition-all duration-150 cursor-pointer border-none disabled:opacity-50 disabled:cursor-not-allowed min-h-11">
          {#if loading}
            Working…
          {:else if mode === 'signin'}
            <LogIn size={16} />
            Sign In
          {:else}
            <UserPlus size={16} />
            Sign Up
          {/if}
        </button>
      </form>

      <p class="mt-5 text-[0.8125rem] text-text-3">
        {mode === 'signin' ? "Don't have an account?" : 'Already have an account?'}
        <button class="bg-transparent border-none text-lime cursor-pointer font-semibold text-[0.8125rem] underline p-0 hover:text-lime-hover" onclick={() => { mode = mode === 'signin' ? 'signup' : 'signin'; error = ''; }}>
          {mode === 'signin' ? 'Sign Up' : 'Sign In'}
        </button>
      </p>
    </div>
  </div>
</div>
