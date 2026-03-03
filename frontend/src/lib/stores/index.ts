import { writable } from 'svelte/store';

// Auth state
export const user = writable<any>(null);
export const isAuthenticated = writable(false);

// Coin balance
export const coinBalance = writable(0);

// Notifications
export const notifications = writable<any[]>([]);
export const unreadCount = writable(0);

// WebSocket connection
export const wsConnected = writable(false);
