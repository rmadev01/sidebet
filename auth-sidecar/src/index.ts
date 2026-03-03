import { betterAuth } from "better-auth";
import { Hono } from "hono";
import { cors } from "hono/cors";
import { serve } from "@hono/node-server";
import pg from "pg";

const pool = new pg.Pool({
    connectionString: process.env.DATABASE_URL || "postgresql://rmadev:password@localhost:5432/p2p_bets",
});

const auth = betterAuth({
    database: pool,
    baseURL: process.env.BETTER_AUTH_URL || "http://localhost:3001",
    secret: process.env.AUTH_SECRET || "dev-secret-change-me",
    emailAndPassword: {
        enabled: true,
    },
    socialProviders: {
        google: {
            clientId: process.env.GOOGLE_CLIENT_ID || "",
            clientSecret: process.env.GOOGLE_CLIENT_SECRET || "",
            enabled: !!(process.env.GOOGLE_CLIENT_ID && process.env.GOOGLE_CLIENT_SECRET),
        },
        discord: {
            clientId: process.env.DISCORD_CLIENT_ID || "",
            clientSecret: process.env.DISCORD_CLIENT_SECRET || "",
            enabled: !!(process.env.DISCORD_CLIENT_ID && process.env.DISCORD_CLIENT_SECRET),
        },
    },
    session: {
        expiresIn: 60 * 60 * 24 * 7, // 7 days
        updateAge: 60 * 60 * 24,      // Update session age every day
    },
    trustedOrigins: [
        "http://localhost:5173",  // SvelteKit dev
        "http://localhost:3000",  // Axum backend
    ],
});

const app = new Hono();

// CORS for frontend
app.use("*", cors({
    origin: ["http://localhost:5173", "http://localhost:3000"],
    credentials: true,
}));

// Health check
app.get("/health", (c) => c.json({ status: "ok" }));

// Mount Better Auth on /api/auth/*
app.on(["GET", "POST"], "/api/auth/**", (c) => {
    return auth.handler(c.req.raw);
});

const port = parseInt(process.env.AUTH_PORT || "3001");

serve({ fetch: app.fetch, port }, () => {
    console.log(`🔐 Auth sidecar running on http://localhost:${port}`);
});

