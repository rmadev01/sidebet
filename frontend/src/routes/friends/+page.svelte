<script lang="ts">
  let searchQuery = $state('');

  const friends = [
    { id: '1', username: 'chad_bets', name: 'Chad', wins: 15, losses: 8, av: 1 },
    { id: '2', username: 'crypto_mike', name: 'Mike', wins: 22, losses: 12, av: 2 },
    { id: '3', username: 'hoops_fan', name: 'Sarah', wins: 9, losses: 5, av: 3 },
    { id: '4', username: 'dub_nation', name: 'Jay', wins: 11, losses: 11, av: 4 },
    { id: '5', username: 'poly_whale', name: 'Alex', wins: 18, losses: 14, av: 5 },
  ];

  const requests = [
    { id: 'r1', username: 'new_player', name: 'Jordan', av: 6 },
  ];

  let filtered = $derived(
    searchQuery
      ? friends.filter(f => f.username.includes(searchQuery.toLowerCase()) || f.name.toLowerCase().includes(searchQuery.toLowerCase()))
      : friends
  );
</script>

<svelte:head><title>SideBet — Friends</title></svelte:head>

<div class="friends-page">
  <h1 class="animate-in">Friends</h1>
  <p class="subtitle animate-in" style="animation-delay:40ms">Manage your betting circle</p>

  <div class="search-row animate-in" style="animation-delay:60ms">
    <input class="input" bind:value={searchQuery} placeholder="Search friends…" />
  </div>

  {#if requests.length > 0}
    <section class="section animate-in" style="animation-delay:80ms">
      <h3>Requests <span class="req-count">{requests.length}</span></h3>
      <div class="stagger">
        {#each requests as req}
          <div class="friend-row accent-bar accent-bar--amber">
            <div class="av av-{req.av}">{req.name[0]}</div>
            <div class="f-info">
              <span class="f-name">@{req.username}</span>
              <span class="f-sub">{req.name}</span>
            </div>
            <div class="f-actions">
              <button class="btn btn-accept btn-sm">Accept</button>
              <button class="btn btn-ghost btn-sm">Ignore</button>
            </div>
          </div>
        {/each}
      </div>
    </section>
  {/if}

  <section class="section animate-in" style="animation-delay:120ms">
    <h3>Your Friends</h3>
    <div class="stagger">
      {#each filtered as f}
        <div class="friend-row">
          <div class="av av-{f.av}">{f.name[0]}</div>
          <div class="f-info">
            <span class="f-name">@{f.username}</span>
            <span class="f-sub">{f.wins}W – {f.losses}L</span>
          </div>
          <a href="/bets/new?opponent={f.id}" class="challenge-link">Challenge</a>
        </div>
      {/each}
    </div>
  </section>
</div>

<style>
  .friends-page { max-width: 600px; }
  .friends-page h1 { margin-bottom: 4px; }
  .subtitle { color: var(--text-2); font-size: 0.9375rem; margin-bottom: 24px; }

  .search-row { margin-bottom: 28px; }
  .search-row .input { max-width: 320px; }

  .section { margin-bottom: 32px; }
  .section h3 {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 14px;
  }
  .req-count {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    background: var(--amber-dim);
    color: var(--amber);
    padding: 1px 7px;
    border-radius: var(--r-full);
  }

  .friend-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 0;
    border-bottom: 1px solid var(--border);
  }
  .friend-row:last-child { border-bottom: none; }

  .f-info { flex: 1; }
  .f-name {
    display: block;
    font-weight: 600;
    font-size: 0.875rem;
    line-height: 1.3;
  }
  .f-sub { font-size: 0.75rem; color: var(--text-3); }

  .f-actions { display: flex; gap: 6px; }

  .challenge-link {
    font-family: var(--font-display);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--lime);
    text-decoration: none;
    opacity: 0;
    transition: opacity var(--dur-fast);
  }
  .friend-row:hover .challenge-link { opacity: 1; }
</style>
