<script lang="ts">
  let searchQuery = $state('');
  let activeTab = $state<'friends' | 'requests'>('friends');

  const demoFriends = [
    { id: 'f1', username: 'chad_bets', display_name: 'Chad', wins: 15, losses: 8, avatar: null },
    { id: 'f2', username: 'crypto_mike', display_name: 'Mike', wins: 22, losses: 12, avatar: null },
    { id: 'f3', username: 'hoops_fan', display_name: 'Sarah', wins: 9, losses: 5, avatar: null },
    { id: 'f4', username: 'dub_nation', display_name: 'Dub', wins: 18, losses: 18, avatar: null },
    { id: 'f5', username: 'macro_trader', display_name: 'Alex', wins: 30, losses: 14, avatar: null },
  ];

  const demoRequests = [
    { id: 'r1', username: 'new_bettor', display_name: 'Jordan', created_at: '2026-03-01T10:00:00Z' },
    { id: 'r2', username: 'poly_whale', display_name: 'Sam', created_at: '2026-02-28T15:00:00Z' },
  ];

  $effect(() => {
    // Filter friends by search (reactive)
  });

  function filteredFriends() {
    if (!searchQuery) return demoFriends;
    const q = searchQuery.toLowerCase();
    return demoFriends.filter(f =>
      f.username.toLowerCase().includes(q) || f.display_name.toLowerCase().includes(q)
    );
  }
</script>

<svelte:head>
  <title>SideBet — Friends</title>
</svelte:head>

<div class="friends-page">
  <div class="page-header animate-slide-up">
    <h1>Friends</h1>
    <p class="text-secondary">Manage your betting circle</p>
  </div>

  <div class="tabs animate-slide-up" style="animation-delay: 60ms">
    <button class="tab" class:active={activeTab === 'friends'} onclick={() => activeTab = 'friends'}>
      Friends ({demoFriends.length})
    </button>
    <button class="tab" class:active={activeTab === 'requests'} onclick={() => activeTab = 'requests'}>
      Requests ({demoRequests.length})
    </button>
  </div>

  {#if activeTab === 'friends'}
    <div class="search-bar animate-slide-up" style="animation-delay: 120ms">
      <input class="input" type="text" placeholder="Search friends..." bind:value={searchQuery} />
    </div>

    <div class="friends-list stagger">
      {#each filteredFriends() as friend}
        <div class="card friend-row">
          <div class="friend-row-left">
            <div class="avatar">{friend.display_name[0]}</div>
            <div class="friend-info">
              <span class="friend-name">{friend.display_name}</span>
              <span class="text-xs text-muted">@{friend.username}</span>
            </div>
          </div>
          <div class="friend-row-right">
            <div class="friend-record">
              <span class="record-win">{friend.wins}W</span>
              <span class="text-muted">-</span>
              <span class="record-loss">{friend.losses}L</span>
            </div>
            <a href="/bets/new?opponent={friend.id}" class="btn btn-primary btn-sm">Send Bet</a>
            <button class="btn btn-ghost btn-sm">⋯</button>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="stagger">
      {#each demoRequests as req}
        <div class="card friend-row">
          <div class="friend-row-left">
            <div class="avatar">{req.display_name[0]}</div>
            <div class="friend-info">
              <span class="friend-name">{req.display_name}</span>
              <span class="text-xs text-muted">@{req.username}</span>
            </div>
          </div>
          <div class="friend-row-right">
            <button class="btn btn-success btn-sm">Accept</button>
            <button class="btn btn-ghost btn-sm">Decline</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .friends-page { max-width: 800px; }
  .page-header { margin-bottom: var(--space-lg); }
  .page-header h1 { margin-bottom: 4px; }

  .search-bar { margin-bottom: var(--space-lg); }

  .friends-list { display: flex; flex-direction: column; gap: var(--space-sm); }

  .friend-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md) var(--space-lg);
    gap: var(--space-md);
  }

  .friend-row-left {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }

  .friend-info { display: flex; flex-direction: column; }
  .friend-name { font-weight: 600; font-size: 0.9375rem; }

  .friend-row-right {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .friend-record {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 0.8125rem;
    font-weight: 600;
    font-family: var(--font-mono);
    margin-right: var(--space-sm);
  }
  .record-win { color: var(--accent-green); }
  .record-loss { color: var(--accent-red); }

  @media (max-width: 640px) {
    .friend-row { flex-direction: column; align-items: flex-start; }
    .friend-row-right { margin-top: var(--space-sm); }
  }
</style>
