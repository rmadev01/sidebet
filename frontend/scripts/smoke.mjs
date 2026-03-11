const required = [
  'PUBLIC_API_URL',
  'PUBLIC_AUTH_URL'
];

const missing = required.filter((name) => !process.env[name]);

if (missing.length > 0) {
  console.error(`Missing required env vars: ${missing.join(', ')}`);
  process.exit(1);
}

console.log('Smoke config check passed');
