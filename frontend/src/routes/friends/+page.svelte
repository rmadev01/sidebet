<script lang="ts">
  import { onMount } from 'svelte';
  import { isAuthenticated } from '$lib/stores';
  import { getFriends, getFriendRequests, searchUsers, sendFriendRequest, acceptFriend, declineFriend, removeFriend } from '$lib/api';

  let friends = $state<any[]>([]);
  let requests = $state<any[]>([]);
  let searchQuery = $state('');
  let searchResults = $state<any[]>([]);
  let searching = $state(false);
  let loading = $state(true);

  onMount(async () => {
    if (!$isAuthenticated) { loading = false; return; }
    try {
      const [f, r] = await Promise.all([
        getFriends().catch(() => []),
        getFriendRequests().catch(() => []),
      ]);
      friends = f;
      requests = r;
    } catch { /* ignore */ }
    loading = false;
  });

  async function doSearch() {
    if (searchQuery.length < 2) { searchResults = []; return; }
    searching = true;
    try {
      searchResults = await searchUsers(searchQuery);
    } catch { searchResults = []; }
    searching = false;
  }

  async function doSendRequest(userId: string) {
    try {
      await sendFriendRequest(userId);
      searchResults = searchResults.filter(u => u.id !== userId);
    } catch { /* ignore */ }
  }

  async function doAccept(id: string) {
    try {
      await acceptFriend(id);
      const req = requests.find(r => r.id === id);
      requests = requests.filter(r => r.id !== id);
      if (req) friends = [...friends, req];
    } catch { /* ignore */ }
  }

  async function doDecline(id: string) {
    try {
      await declineFriend(id);
      requests = requests.filter(r => r.id !== id);
    } catch { /* ignore */ }
  }

  async function doRemove(id: string) {
    try {
      await removeFriend(id);
      friends = friends.filter(f => f.id !== id);
    } catch { /* ignore */ }
  }
</script>

<svelte:head>
  <title>Friends — SideBet</title>
</svelte:head>

<div class="friends-page">
  <h1>Friends</h1>

  <!-- Search -->
  <div class="search-bar animate-in">
    <input
      type="text"
      class="input"
      placeholder="Search users by name or username…"
      bind:value={searchQuery}
      oninput={doSearch}
    />
    {#if searchResults.length > 0}
      <div class="search-results">
        {#each searchResults as u}
          <div class="search-row">
            <span class="av av-1 av--sm">{u.display_name?.[0]?.toUpperCase() || '?'}</span>
            <div class="search-info">
              <span class="search-name">{u.display_name}</span>
              <span class="search-handle">@{u.username}</span>
            </div>
            <button class="btn btn-primary btn-sm" onclick={() => doSendRequest(u.id)}>Add</button>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Pending Requests -->
  {#if requests.length > 0}
    <section class="section animate-in" style="animation-delay: 80ms">
      <h2>Pending Requests <span class="count">{requests.length}</span></h2>
      <div class="friend-list">
        {#each requests as req}
          <div class="friend-row">
            <span class="av av-2 av--sm">{req.display_name?.[0] || '?'}</span>
            <div class="friend-info">
              <span class="friend-name">{req.display_name || req.username || 'User'}</span>
              <span class="friend-handle">@{req.username || '...'}</span>
            </div>
            <div class="friend-actions">
              <button class="btn btn-primary btn-sm" onclick={() => doAccept(req.id)}>Accept</button>
              <button class="btn btn-ghost btn-sm" onclick={() => doDecline(req.id)}>Ignore</button>
            </div>
          </div>
        {/each}
      </div>
    </section>
  {/if}

  <!-- Friends -->
  <section class="section animate-in" style="animation-delay: 160ms">
    <h2>Your Friends <span class="count">{friends.length}</span></h2>
    {#if friends.length === 0}
      <p class="empty-state">No friends yet. Search above to add people!</p>
    {:else}
      <div class="friend-list">
        {#each friends as f}
          <div class="friend-row">
            <span class="av av-1 av--sm">{f.display_name?.[0]?.toUpperCase() || '?'}</span>
            <div class="friend-info">
              <span class="friend-name">{f.display_name || f.username}</span>
              <span class="friend-handle">@{f.username}</span>
            </div>
            <div class="friend-actions">
              <a href="/bets/new?opponent={f.id}" class="btn btn-primary btn-sm">Challenge</a>
              <button class="btn btn-ghost btn-sm" onclick={() => doRemove(f.id)}>Remove</button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </section>
</div>

<style>
  .friends-page { max-width: 640px; }

  h1 { font-size: 1.5rem; margin-bottom: 24px; }

  .search-bar {
    position: relative;
    margin-bottom: 32px;
  }
  .search-results {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-top: none;
    border-radius: 0 0 var(--r-md) var(--r-md);
    max-height: 240px;
    overflow-y: auto;
    z-index: 50;
  }
  .search-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--border);
  }
  .search-row:last-child { border-bottom: none; }
  .search-info { flex: 1; min-width: 0; }
  .search-name { display: block; font-weight: 600; font-size: 0.875rem; }
  .search-handle { font-size: 0.75rem; color: var(--text-3); }

  .section { margin-bottom: 32px; }
  .section h2 {
    font-size: 1rem;
    margin-bottom: 12px;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .count {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--text-3);
    background: var(--bg-raised);
    padding: 2px 8px;
    border-radius: var(--r-full);
  }

  .empty-state {
    color: var(--text-3);
    font-size: 0.875rem;
    padding: 20px 0;
  }

  .friend-list {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    overflow: hidden;
  }
  .friend-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
  }
  .friend-row:last-child { border-bottom: none; }
  .friend-info { flex: 1; min-width: 0; }
  .friend-name { display: block; font-weight: 600; font-size: 0.9375rem; }
  .friend-handle { font-size: 0.75rem; color: var(--text-3); }
  .friend-actions { display: flex; gap: 6px; flex-shrink: 0; }
</style>
