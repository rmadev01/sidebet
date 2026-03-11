import { get } from 'svelte/store';

import { authReady, isAuthenticated } from '$lib/stores';

export function runWhenAuthResolved(
	loadAuthenticated: () => void | Promise<void>,
	onGuest?: () => void | Promise<void>
) {
	let started = false;

	const maybeRun = () => {
		if (!get(authReady) || started) {
			return;
		}

		started = true;
		if (get(isAuthenticated)) {
			void loadAuthenticated();
		} else {
			void onGuest?.();
		}
	};

	const unsubscribeReady = authReady.subscribe(() => {
		maybeRun();
	});
	const unsubscribeAuth = isAuthenticated.subscribe(() => {
		maybeRun();
	});

	maybeRun();

	return () => {
		unsubscribeReady();
		unsubscribeAuth();
	};
}
