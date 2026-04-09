<script lang="ts">
  import { page } from '$app/stores';
  import { AlertTriangle } from 'lucide-svelte';
</script>

<svelte:head>
  <title>Error {$page.status} — SideBet</title>
</svelte:head>

<div class="flex flex-col items-center justify-center text-center py-20 animate-enter">
  <div class="w-16 h-16 rounded-2xl bg-rose-dim text-rose flex items-center justify-center mb-6">
    <AlertTriangle size={28} strokeWidth={2} />
  </div>

  <h1 class="text-5xl font-display tracking-tight mb-2">{$page.status}</h1>

  <p class="text-lg text-text-2 max-w-md mb-8">
    {#if $page.status === 404}
      The page you're looking for doesn't exist or has been moved.
    {:else}
      {$page.error?.message || 'Something went wrong. Please try again.'}
    {/if}
  </p>

  <div class="flex items-center gap-3">
    <a href="/" class="inline-flex items-center gap-2 px-5 py-2.5 rounded-2xl bg-lime text-white font-display text-sm font-semibold no-underline shadow-[0_12px_28px_rgba(31,122,104,0.18)] hover:bg-lime-hover transition-colors">
      Back to Dashboard
    </a>
  </div>
</div>
