import { PUBLIC_API_URL, PUBLIC_AUTH_URL } from '$env/static/public';

const API_BASE = PUBLIC_API_URL;
const AUTH_BASE = PUBLIC_AUTH_URL;

export type PublicUser = {
	id: string;
	username: string;
	display_name: string;
	avatar_url?: string | null;
	bio?: string | null;
	wins?: number;
	losses?: number;
};

export type FriendSummary = PublicUser & {
	friendship_id: string;
	user_id: string;
};

export type PendingFriendRequest = PublicUser & {
	friendship_id: string;
	requester_id: string;
	created_at?: string;
};

export type AuthSessionUser = {
	id: string;
	name: string;
	email: string;
	image?: string | null;
};

export type AuthSession = {
	user: AuthSessionUser;
	session: {
		id: string;
		expiresAt: string;
	};
};

export type AppUser = {
	id: string;
	username: string;
	display_name: string;
	avatar_url?: string | null;
	bio?: string | null;
	coin_balance: number;
	wins: number;
	losses: number;
	total_wagered: number;
	created_at: string;
};

export type WalletBalance = {
	coin_balance: number;
	last_daily_bonus?: string | null;
	bonus_available: boolean;
};

export type EventSummary = {
	id: string;
	title: string;
	category: string;
	description?: string | null;
	sport?: string | null;
	league?: string | null;
	starts_at?: string | null;
	status: string;
};

export type Bet = {
	id: string;
	creator_id: string;
	opponent_id?: string | null;
	event_id?: string | null;
	question: string;
	creator_position: string;
	opponent_position: string;
	amount: number;
	odds_numerator: number;
	odds_denominator: number;
	status: string;
	winner_id?: string | null;
	outcome?: string | null;
	resolved_at?: string | null;
	expires_at: string;
	created_at: string;
	updated_at: string;
};

export type OpenBet = Bet & {
	creator_username: string;
	creator_display_name: string;
};

export type FeedItem = {
	id: string;
	type: string;
	payload: Record<string, any>;
	created_at: string;
};

export type Transaction = {
	id: string;
	type: string;
	amount: number;
	balance_after: number;
	created_at: string;
};

export type DailyBonusResponse = {
	coins_awarded: number;
	new_balance: number;
};

function buildHeaders(options: RequestInit) {
	const headers = new Headers(options.headers);
	const method = (options.method ?? 'GET').toUpperCase();
	const hasBody = options.body !== undefined && options.body !== null;

	if (hasBody && method !== 'GET' && !headers.has('Content-Type')) {
		headers.set('Content-Type', 'application/json');
	}

	return headers;
}

async function apiFetch<T = any>(path: string, options: RequestInit = {}): Promise<T> {
	const res = await fetch(`${API_BASE}${path}`, {
		credentials: 'include',
		...options,
		headers: buildHeaders(options)
	});

	if (!res.ok) {
		const text = await res.text();
		throw new Error(`API error ${res.status}: ${text}`);
	}

	if (res.status === 204) {
		return null as T;
	}

	const contentType = res.headers.get('content-type') || '';
	if (contentType.includes('application/json')) {
		return res.json();
	}

	return res.text() as Promise<T>;
}

export const getMe = () => apiFetch<AppUser>('/api/users/me');
export const updateMe = (data: Partial<Pick<AppUser, 'display_name' | 'bio' | 'avatar_url'>>) =>
	apiFetch<AppUser>('/api/users/me', { method: 'PATCH', body: JSON.stringify(data) });
export const getUser = (username: string) => apiFetch<PublicUser>(`/api/users/${username}`);
export const searchUsers = (q: string, signal?: AbortSignal) =>
	apiFetch<PublicUser[]>(`/api/users/search?q=${encodeURIComponent(q)}`, { signal });

export const getFriends = () => apiFetch<FriendSummary[]>('/api/friends');
export const getFriendRequests = () => apiFetch<PendingFriendRequest[]>('/api/friends/requests');
export const sendFriendRequest = (user_id: string) =>
	apiFetch('/api/friends/request', { method: 'POST', body: JSON.stringify({ user_id }) });
export const acceptFriend = (id: string) => apiFetch(`/api/friends/${id}/accept`, { method: 'POST' });
export const declineFriend = (id: string) => apiFetch(`/api/friends/${id}/decline`, { method: 'POST' });
export const removeFriend = (id: string) => apiFetch(`/api/friends/${id}`, { method: 'DELETE' });

	export const getEvents = (params?: { category?: string; league?: string; status?: string }) => {
	const qs = new URLSearchParams(params as any).toString();
	return apiFetch<EventSummary[]>(`/api/events${qs ? '?' + qs : ''}`);
};
export const getEvent = (id: string) => apiFetch<EventSummary>(`/api/events/${id}`);
export const getEventOdds = (id: string) => apiFetch(`/api/events/${id}/odds`);
export const syncEvents = (leagues?: string) => {
	const qs = leagues ? `?leagues=${encodeURIComponent(leagues)}` : '';
	return apiFetch(`/api/events/sync${qs}`, { method: 'POST' });
};

export const createBet = (data: Record<string, any>) =>
	apiFetch<Bet>('/api/bets', { method: 'POST', body: JSON.stringify(data) });
export const getBets = (params?: { status?: string; role?: string }) => {
	const qs = new URLSearchParams(params as any).toString();
	return apiFetch<Bet[]>(`/api/bets${qs ? '?' + qs : ''}`);
};
export const getBet = (id: string) => apiFetch<Bet>(`/api/bets/${id}`);
export const acceptBet = (id: string) => apiFetch<Bet>(`/api/bets/${id}/accept`, { method: 'POST' });
export const declineBet = (id: string) => apiFetch(`/api/bets/${id}/decline`, { method: 'POST' });
export const cancelBet = (id: string) => apiFetch(`/api/bets/${id}/cancel`, { method: 'POST' });
export const settleBet = (id: string, outcome: string) =>
	apiFetch<Bet>(`/api/bets/${id}/settle`, { method: 'POST', body: JSON.stringify({ outcome }) });

export const getFeed = () => apiFetch<FeedItem[]>('/api/feed');
export const getOpenBets = () => apiFetch<OpenBet[]>('/api/feed/open');
export const takeBet = (id: string) => apiFetch<Bet>(`/api/bets/${id}/take`, { method: 'POST' });

export const getBalance = () => apiFetch<WalletBalance>('/api/wallet/balance');
export const claimDailyBonus = () => apiFetch<DailyBonusResponse>('/api/wallet/daily-bonus', { method: 'POST' });
export const getTransactions = (limit?: number) =>
	apiFetch<Transaction[]>(`/api/wallet/transactions${limit ? '?limit=' + limit : ''}`);

export const getNotifications = () => apiFetch('/api/notifications');
export const markNotificationsRead = () => apiFetch('/api/notifications/read', { method: 'POST' });

export const signUp = (email: string, password: string, name: string) =>
	fetch(`${AUTH_BASE}/api/auth/sign-up/email`, {
		method: 'POST',
		credentials: 'include',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ email, password, name })
	}).then((r) => r.json());

export const signIn = (email: string, password: string) =>
	fetch(`${AUTH_BASE}/api/auth/sign-in/email`, {
		method: 'POST',
		credentials: 'include',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ email, password })
	}).then((r) => r.json());

export const signOut = () =>
	fetch(`${AUTH_BASE}/api/auth/sign-out`, { method: 'POST', credentials: 'include' });

export const getSession = () =>
	fetch(`${AUTH_BASE}/api/auth/get-session`, { credentials: 'include' }).then((r) =>
		r.ok ? (r.json() as Promise<AuthSession>) : null
	);
