const API_BASE = 'http://localhost:3000';

async function apiFetch(path: string, options: RequestInit = {}) {
    const res = await fetch(`${API_BASE}${path}`, {
        credentials: 'include',
        headers: { 'Content-Type': 'application/json', ...options.headers as any },
        ...options,
    });
    if (!res.ok) {
        const text = await res.text();
        throw new Error(`API error ${res.status}: ${text}`);
    }
    return res.json();
}

// ── Users ──
export const getMe = () => apiFetch('/api/users/me');
export const updateMe = (data: any) => apiFetch('/api/users/me', { method: 'PATCH', body: JSON.stringify(data) });
export const getUser = (username: string) => apiFetch(`/api/users/${username}`);
export const searchUsers = (q: string) => apiFetch(`/api/users/search?q=${encodeURIComponent(q)}`);

// ── Friends ──
export const getFriends = () => apiFetch('/api/friends');
export const getFriendRequests = () => apiFetch('/api/friends/requests');
export const sendFriendRequest = (user_id: string) => apiFetch('/api/friends/request', { method: 'POST', body: JSON.stringify({ user_id }) });
export const acceptFriend = (id: string) => apiFetch(`/api/friends/${id}/accept`, { method: 'POST' });
export const declineFriend = (id: string) => apiFetch(`/api/friends/${id}/decline`, { method: 'POST' });
export const removeFriend = (id: string) => apiFetch(`/api/friends/${id}`, { method: 'DELETE' });

// ── Events ──
export const getEvents = (params?: { category?: string; status?: string }) => {
    const qs = new URLSearchParams(params as any).toString();
    return apiFetch(`/api/events${qs ? '?' + qs : ''}`);
};
export const getEvent = (id: string) => apiFetch(`/api/events/${id}`);
export const getEventOdds = (id: string) => apiFetch(`/api/events/${id}/odds`);

// ── Bets ──
export const createBet = (data: any) => apiFetch('/api/bets', { method: 'POST', body: JSON.stringify(data) });
export const getBets = (params?: { status?: string; role?: string }) => {
    const qs = new URLSearchParams(params as any).toString();
    return apiFetch(`/api/bets${qs ? '?' + qs : ''}`);
};
export const getBet = (id: string) => apiFetch(`/api/bets/${id}`);
export const acceptBet = (id: string) => apiFetch(`/api/bets/${id}/accept`, { method: 'POST' });
export const declineBet = (id: string) => apiFetch(`/api/bets/${id}/decline`, { method: 'POST' });
export const cancelBet = (id: string) => apiFetch(`/api/bets/${id}/cancel`, { method: 'POST' });
export const settleBet = (id: string) => apiFetch(`/api/bets/${id}/settle`, { method: 'POST' });

// ── Feed ──
export const getFeed = () => apiFetch('/api/feed');

// ── Auth (via Better Auth sidecar) ──
const AUTH_BASE = 'http://localhost:3001';

export const signUp = (email: string, password: string, name: string) =>
    fetch(`${AUTH_BASE}/api/auth/sign-up`, {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password, name }),
    }).then(r => r.json());

export const signIn = (email: string, password: string) =>
    fetch(`${AUTH_BASE}/api/auth/sign-in/email`, {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password }),
    }).then(r => r.json());

export const signOut = () =>
    fetch(`${AUTH_BASE}/api/auth/sign-out`, { method: 'POST', credentials: 'include' });

export const getSession = () =>
    fetch(`${AUTH_BASE}/api/auth/get-session`, { credentials: 'include' }).then(r => r.ok ? r.json() : null);
