<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { authReady, coinBalance, isAuthenticated, user } from '$lib/stores';
  import { signIn, signUp, getSession, getBalance } from '$lib/api';
  import { LogIn, UserPlus, Coins, ShieldCheck, Users } from 'lucide-svelte';

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

<div class="min-h-[calc(100vh-var(--nav-height,56px)-120px)] grid lg:grid-cols-[minmax(0,1.05fr)_430px] gap-6 items-stretch">
  <section class="panel rounded-[36px] p-7 sm:p-8 animate-enter overflow-hidden relative">
    <div class="absolute inset-x-0 top-0 h-32 bg-[linear-gradient(135deg,rgba(31,122,104,0.16),rgba(248,195,106,0.16))]"></div>
    <div class="relative">
      <div class="section-kicker mb-4">Join the Rivalry</div>
      <h1 class="font-display text-4xl max-sm:text-[2rem] tracking-tight max-w-xl text-balance">
        {mode === 'signin' ? 'Step back into the action.' : 'Set up your circle and start talking odds.'}
      </h1>
      <p class="mt-4 text-text-2 max-w-xl text-balance">
        SideBet is built for friend groups, league debates, and casual receipts. Every account gets a virtual coin stack to start.
      </p>

      <img src="/sidebet-login-scene.svg" alt="SideBet account preview" class="mt-7 w-full rounded-[28px] border border-white/30 bg-[#173230]" />

      <div class="grid sm:grid-cols-3 gap-3 mt-6">
        <div class="panel-soft rounded-2xl p-4">
          <div class="w-10 h-10 rounded-2xl bg-lime-dim text-lime flex items-center justify-center mb-3"><Coins size={18} /></div>
          <div class="font-display text-sm">10,000 starter coins</div>
          <div class="text-xs text-text-3 mt-1">Plenty of room to test takes and keep score.</div>
        </div>
        <div class="panel-soft rounded-2xl p-4">
          <div class="w-10 h-10 rounded-2xl bg-amber-dim text-amber flex items-center justify-center mb-3"><Users size={18} /></div>
          <div class="font-display text-sm">Direct or public bets</div>
          <div class="text-xs text-text-3 mt-1">Challenge one friend or throw it into the room.</div>
        </div>
        <div class="panel-soft rounded-2xl p-4">
          <div class="w-10 h-10 rounded-2xl bg-sky-dim text-sky flex items-center justify-center mb-3"><ShieldCheck size={18} /></div>
          <div class="font-display text-sm">No real-money risk</div>
          <div class="text-xs text-text-3 mt-1">All gameplay stays inside the SideBet coin economy.</div>
        </div>
      </div>
    </div>
  </section>

  <section class="panel rounded-[32px] px-6 py-7 sm:px-8 sm:py-8 animate-enter self-center" style="animation-delay: 80ms">
    <div class="flex items-center gap-3 mb-6">
      <img src="/sidebet-mark.svg" alt="SideBet" class="w-11 h-11 rounded-2xl shadow-[0_10px_24px_rgba(20,38,34,0.14)]" />
      <div>
        <div class="font-display text-[0.78rem] font-bold tracking-[0.24em] text-lime">SIDEBET</div>
        <div class="text-xs text-text-3 mt-1">Auth and wallet sync included.</div>
      </div>
    </div>

    <h2 class="text-[1.45rem] font-display tracking-tight mb-1.5">{mode === 'signin' ? 'Welcome back' : 'Create your account'}</h2>
    <p class="text-[0.875rem] text-text-3 mb-6">
      {mode === 'signin'
        ? 'Use your email and password to pick up where you left off.'
        : 'Start with a lightweight account and jump straight into free coin wagers.'}
    </p>

    <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="text-left">
      {#if mode === 'signup'}
        <div class="mb-4">
          <label for="name" class="block text-xs font-semibold text-text-3 uppercase tracking-wide mb-1.5">Name</label>
          <input id="name" type="text" bind:value={name} placeholder="Your name" required
            class="w-full py-2.5 px-3.5 font-body text-[0.9375rem] bg-input text-text-1 border border-border rounded-xl outline-none transition-all duration-150 focus:border-lime focus:shadow-[0_0_0_2px_var(--color-lime-dim)] placeholder:text-text-3" />
        </div>
      {/if}

      <div class="mb-4">
        <label for="email" class="block text-xs font-semibold text-text-3 uppercase tracking-wide mb-1.5">Email</label>
        <input id="email" type="email" bind:value={email} placeholder="you@example.com" required
          class="w-full py-2.5 px-3.5 font-body text-[0.9375rem] bg-input text-text-1 border border-border rounded-xl outline-none transition-all duration-150 focus:border-lime focus:shadow-[0_0_0_2px_var(--color-lime-dim)] placeholder:text-text-3" />
      </div>

      <div class="mb-4">
        <label for="password" class="block text-xs font-semibold text-text-3 uppercase tracking-wide mb-1.5">Password</label>
        <input id="password" type="password" bind:value={password} placeholder="At least 8 characters" required minlength="8"
          class="w-full py-2.5 px-3.5 font-body text-[0.9375rem] bg-input text-text-1 border border-border rounded-xl outline-none transition-all duration-150 focus:border-lime focus:shadow-[0_0_0_2px_var(--color-lime-dim)] placeholder:text-text-3" />
      </div>

      {#if error}
        <p class="text-rose text-[0.8125rem] mb-3">{error}</p>
      {/if}

      <button type="submit" disabled={loading}
        class="w-full mt-2 inline-flex items-center justify-center gap-1.5 px-5 py-3 font-display text-sm font-semibold bg-lime text-white rounded-2xl hover:bg-lime-hover active:scale-[0.97] transition-all duration-150 cursor-pointer border-none disabled:opacity-50 disabled:cursor-not-allowed min-h-11 shadow-[0_14px_28px_rgba(31,122,104,0.18)]">
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
  </section>
</div>
