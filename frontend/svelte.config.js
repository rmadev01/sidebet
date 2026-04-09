import adapter from '@sveltejs/adapter-node';
import vercel from '@sveltejs/adapter-vercel';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		adapter:
			process.env.DEPLOY_TARGET === 'vercel' ||
			process.env.VERCEL === '1' ||
			typeof process.env.VERCEL_ENV === 'string'
				? vercel({ runtime: 'nodejs22.x' })
				: adapter()
	}
};

export default config;
