import { writable } from 'svelte/store';

import type { AuthSessionUser } from '$lib/api';

// Auth state
export const user = writable<AuthSessionUser | null>(null);
export const isAuthenticated = writable(false);
export const authReady = writable(false);

// Coin balance
export const coinBalance = writable(0);

// Notifications
export const notifications = writable<any[]>([]);
export const unreadCount = writable(0);

// WebSocket connection
export const wsConnected = writable(false);
