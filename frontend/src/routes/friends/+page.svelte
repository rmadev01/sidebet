<script lang="ts">
  import { onMount } from 'svelte';
  import { user } from '$lib/stores';
  import { runWhenAuthResolved } from '$lib/auth';
  import {
    acceptFriend,
    declineFriend,
    getFriendRequests,
    getFriends,
    removeFriend,
    searchUsers,
    sendFriendRequest,
    type FriendSummary,
    type PendingFriendRequest,
    type PublicUser,
  } from '$lib/api';
  import { Search, UserPlus, Swords, UserMinus, Check, X } from 'lucide-svelte';

  let friends = $state<FriendSummary[]>([]);
  let requests = $state<PendingFriendRequest[]>([]);
  let searchQuery = $state('');
  let searchResults = $state<PublicUser[]>([]);
  let searching = $state(false);
  let loading = $state(true);
  let error = $state('');
  let searchDebounce: ReturnType<typeof setTimeout> | null = null;
  let searchController: AbortController | null = null;

  onMount(() => {
    const unsubscribe = runWhenAuthResolved(async () => {
      loading = true;
      error = '';
      try {
        const [f, r] = await Promise.all([
          getFriends().catch(() => []),
          getFriendRequests().catch(() => []),
        ]);
        friends = f;
        requests = r;
      } catch (e: any) {
        friends = [];
        requests = [];
        error = e.message || 'Failed to load friends';
      } finally {
        loading = false;
      }
    }, async () => {
      friends = [];
      requests = [];
      loading = false;
    });

    return () => {
      unsubscribe?.();
      if (searchDebounce) {
        clearTimeout(searchDebounce);
      }
      searchController?.abort();
    };
  });

  function existingUserIds() {
    return new Set([
      $user?.id,
      ...friends.map((friend) => friend.user_id),
      ...requests.map((request) => request.requester_id),
    ].filter(Boolean));
  }

  function visibleSearchResults(results: PublicUser[]) {
    const excludedIds = existingUserIds();
    return results.filter((candidate) => !excludedIds.has(candidate.id));
  }

  async function doSearch() {
    if (searchQuery.trim().length < 2) {
      searchController?.abort();
      searchResults = [];
      searching = false;
      return;
    }

    searching = true;
    searchController?.abort();
    searchController = new AbortController();

    try {
      const results = await searchUsers(searchQuery.trim(), searchController.signal);
      searchResults = visibleSearchResults(results);
    } catch {
      searchResults = [];
    }

    searching = false;
  }

  function queueSearch() {
    if (searchDebounce) {
      clearTimeout(searchDebounce);
    }

    searchDebounce = setTimeout(() => {
      void doSearch();
    }, 250);
  }

  async function doSendRequest(userId: string) {
    error = '';
    try {
      const result = await sendFriendRequest(userId);
      searchResults = searchResults.filter((candidate) => candidate.id !== userId);
      searchQuery = '';
      if (result?.status === 'accepted' && result?.user) {
        friends = [
          {
            ...result.user,
            friendship_id: result.friendship_id,
            user_id: result.user.id,
          },
          ...friends,
        ].sort((a, b) => a.username.localeCompare(b.username));
      }
    } catch (e: any) {
      error = e.message || 'Failed to send friend request';
    }
  }

  async function doAccept(friendshipId: string) {
    error = '';
    try {
      const acceptedFriend = await acceptFriend(friendshipId);
      requests = requests.filter((request) => request.friendship_id !== friendshipId);
      friends = [...friends, acceptedFriend].sort((a, b) => a.username.localeCompare(b.username));
    } catch (e: any) {
      error = e.message || 'Failed to accept friend request';
    }
  }

  async function doDecline(friendshipId: string) {
    error = '';
    try {
      await declineFriend(friendshipId);
      requests = requests.filter((request) => request.friendship_id !== friendshipId);
    } catch (e: any) {
      error = e.message || 'Failed to decline friend request';
    }
  }

  async function doRemove(friendshipId: string) {
    error = '';
    try {
      await removeFriend(friendshipId);
      friends = friends.filter((friend) => friend.friendship_id !== friendshipId);
    } catch (e: any) {
      error = e.message || 'Failed to remove friend';
    }
  }
</script>

<svelte:head>
  <title>Friends — SideBet</title>
</svelte:head>

<div class="max-w-2xl">
  <h1 class="text-2xl font-display tracking-tight mb-6">Friends</h1>

  <div class="relative mb-8 animate-enter">
    <div class="relative">
      <Search size={16} class="absolute left-3 top-1/2 -translate-y-1/2 text-text-3" />
      <input
        type="text"
        class="w-full py-2.5 pl-9 pr-3.5 font-body text-[0.9375rem] bg-input text-text-1 border border-border rounded-md outline-none transition-all duration-150 focus:border-lime focus:shadow-[0_0_0_2px_var(--color-lime-dim)] placeholder:text-text-3"
        placeholder="Search users by name or username…"
        bind:value={searchQuery}
        oninput={queueSearch}
      />
    </div>
    {#if searching}
      <p class="mt-2 text-xs text-text-3">Searching…</p>
    {/if}
    {#if searchResults.length > 0}
      <div class="absolute top-full left-0 right-0 bg-surface border border-border border-t-0 rounded-b-md max-h-60 overflow-y-auto z-50 shadow-lg">
        {#each searchResults as candidate}
          <div class="flex items-center gap-3 px-3.5 py-2.5 border-b border-border last:border-b-0">
            <span class="w-[26px] h-[26px] rounded-full flex items-center justify-center font-display font-bold text-[0.625rem] text-white bg-lime shrink-0">{candidate.display_name?.[0]?.toUpperCase() || '?'}</span>
            <div class="flex-1 min-w-0">
              <span class="block font-semibold text-sm">{candidate.display_name}</span>
              <span class="text-xs text-text-3">@{candidate.username}</span>
            </div>
            <button class="inline-flex items-center justify-center gap-1 px-3 py-1.5 font-display text-xs font-semibold bg-lime text-white rounded-md hover:bg-lime-hover active:scale-95 transition-all duration-150 cursor-pointer border-none" onclick={() => doSendRequest(candidate.id)}>
              <UserPlus size={12} />
              Add
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  {#if error}
    <p class="text-rose text-sm mb-4">{error}</p>
  {/if}

  {#if requests.length > 0}
    <section class="mb-8 animate-enter" style="animation-delay: 80ms">
      <h2 class="text-base font-display flex items-center gap-2 mb-3">
        Pending Requests
        <span class="font-mono text-xs text-text-3 bg-raised px-2 py-0.5 rounded-full">{requests.length}</span>
      </h2>
      <div class="bg-surface border border-border rounded-md overflow-hidden">
        {#each requests as req}
          <div class="flex items-center gap-3 px-4 py-3 border-b border-border last:border-b-0">
            <span class="w-[26px] h-[26px] rounded-full flex items-center justify-center font-display font-bold text-[0.625rem] text-white bg-rose shrink-0">{req.display_name?.[0] || '?'}</span>
            <div class="flex-1 min-w-0">
              <span class="block font-semibold text-[0.9375rem]">{req.display_name || req.username || 'User'}</span>
              <span class="text-xs text-text-3">@{req.username || '...'}</span>
            </div>
            <div class="flex gap-1.5 shrink-0">
              <button class="inline-flex items-center justify-center gap-1 px-3 py-1.5 font-display text-xs font-semibold bg-lime text-white rounded-md hover:bg-lime-hover active:scale-95 transition-all duration-150 cursor-pointer border-none" onclick={() => doAccept(req.friendship_id)}>
                <Check size={12} />
                Accept
              </button>
              <button class="inline-flex items-center justify-center gap-1 px-3 py-1.5 font-display text-xs font-semibold bg-transparent text-text-2 border border-border rounded-md hover:bg-raised hover:text-text-1 active:scale-95 transition-all duration-150 cursor-pointer" onclick={() => doDecline(req.friendship_id)}>
                <X size={12} />
                Ignore
              </button>
            </div>
          </div>
        {/each}
      </div>
    </section>
  {/if}

  <section class="mb-8 animate-enter" style="animation-delay: 160ms">
    <h2 class="text-base font-display flex items-center gap-2 mb-3">
      Your Friends
      <span class="font-mono text-xs text-text-3 bg-raised px-2 py-0.5 rounded-full">{friends.length}</span>
    </h2>
    {#if loading}
      <p class="text-text-3 text-sm py-5">Loading friends…</p>
    {:else if friends.length === 0}
      <p class="text-text-3 text-sm py-5">No friends yet. Search above to add people.</p>
    {:else}
      <div class="bg-surface border border-border rounded-md overflow-hidden">
        {#each friends as friend}
          <div class="flex items-center gap-3 px-4 py-3 border-b border-border last:border-b-0 hover:bg-raised/30 transition-colors duration-150">
            <span class="w-[26px] h-[26px] rounded-full flex items-center justify-center font-display font-bold text-[0.625rem] text-white bg-lime shrink-0">{friend.display_name?.[0]?.toUpperCase() || '?'}</span>
            <div class="flex-1 min-w-0">
              <span class="block font-semibold text-[0.9375rem]">{friend.display_name || friend.username}</span>
              <span class="text-xs text-text-3">@{friend.username}</span>
            </div>
            <div class="flex gap-1.5 shrink-0">
              <a href="/bets/new?opponent={friend.user_id}" class="inline-flex items-center justify-center gap-1 px-3 py-1.5 font-display text-xs font-semibold bg-lime text-white rounded-md no-underline hover:bg-lime-hover active:scale-95 transition-all duration-150">
                <Swords size={12} />
                Challenge
              </a>
              <button class="inline-flex items-center justify-center gap-1 px-3 py-1.5 font-display text-xs font-semibold bg-transparent text-text-2 border border-border rounded-md hover:bg-raised hover:text-text-1 active:scale-95 transition-all duration-150 cursor-pointer" onclick={() => doRemove(friend.friendship_id)}>
                <UserMinus size={12} />
                Remove
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </section>
</div>
