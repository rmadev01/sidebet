import "dotenv/config";
import { serve } from "@hono/node-server";
import { betterAuth } from "better-auth";
import { cors } from "hono/cors";
import { Hono } from "hono";
import pg from "pg";

type AppEnv = "development" | "production";

function requireEnv(name: string): string {
    const value = process.env[name]?.trim();
    if (!value) {
        throw new Error(`${name} must be set`);
    }

    return value;
}

function parseIntEnv(name: string, fallback: number): number {
    const raw = process.env[name];
    if (!raw) {
        return fallback;
    }

    const parsed = Number.parseInt(raw, 10);
    if (Number.isNaN(parsed) || parsed <= 0) {
        throw new Error(`${name} must be a positive integer`);
    }

    return parsed;
}

function parseAppEnv(): AppEnv {
    const value = (process.env.APP_ENV || "development").toLowerCase();
    return value === "production" || value === "prod" ? "production" : "development";
}

function splitOrigins(value: string): string[] {
    return value
        .split(",")
        .map((origin) => origin.trim())
        .filter(Boolean);
}

function allowedFrontendOrigins(appEnv: AppEnv, configuredValue: string): string[] {
    const configured = splitOrigins(configuredValue);
    if (configured.length > 0) {
        return configured;
    }

    if (appEnv === "production") {
        throw new Error("FRONTEND_URL must be set in production");
    }

    return [
        "http://localhost:5173",
        "http://localhost:5174",
        "http://localhost:5175",
        "http://127.0.0.1:5173",
        "http://127.0.0.1:5174",
        "http://127.0.0.1:5175",
    ];
}

function rateLimitKey(req: Request): string {
    const forwarded = req.headers.get("x-forwarded-for") || req.headers.get("cf-connecting-ip") || "unknown";
    return `${forwarded.split(",")[0].trim()}:${new URL(req.url).pathname}`;
}

async function ensureAuthSchema(pool: pg.Pool) {
    await pool.query(`
        CREATE TABLE IF NOT EXISTS "user" (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            "emailVerified" BOOLEAN NOT NULL DEFAULT FALSE,
            image TEXT,
            "createdAt" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            "updatedAt" TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    `);

    await pool.query(`
        CREATE TABLE IF NOT EXISTS account (
            id TEXT PRIMARY KEY,
            "accountId" TEXT NOT NULL,
            "providerId" TEXT NOT NULL,
            "userId" TEXT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
            "accessToken" TEXT,
            "refreshToken" TEXT,
            "idToken" TEXT,
            "accessTokenExpiresAt" TIMESTAMPTZ,
            "refreshTokenExpiresAt" TIMESTAMPTZ,
            scope TEXT,
            password TEXT,
            "createdAt" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            "updatedAt" TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    `);

    await pool.query(`
        CREATE UNIQUE INDEX IF NOT EXISTS idx_account_provider_lookup
            ON account ("providerId", "accountId")
    `);
    await pool.query(`
        CREATE INDEX IF NOT EXISTS idx_account_user_id
            ON account ("userId")
    `);

    await pool.query(`
        CREATE TABLE IF NOT EXISTS "session" (
            id TEXT PRIMARY KEY,
            token TEXT NOT NULL UNIQUE,
            "userId" TEXT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
            "expiresAt" TIMESTAMPTZ NOT NULL,
            "ipAddress" TEXT,
            "userAgent" TEXT,
            "createdAt" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            "updatedAt" TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    `);

    await pool.query(`
        CREATE INDEX IF NOT EXISTS idx_session_user_id
            ON "session" ("userId")
    `);
    await pool.query(`
        CREATE INDEX IF NOT EXISTS idx_session_token_expires
            ON "session" (token, "expiresAt")
    `);

    await pool.query(`
        CREATE TABLE IF NOT EXISTS verification (
            id TEXT PRIMARY KEY,
            identifier TEXT NOT NULL,
            value TEXT NOT NULL,
            "expiresAt" TIMESTAMPTZ NOT NULL,
            "createdAt" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            "updatedAt" TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    `);

    await pool.query(`
        CREATE INDEX IF NOT EXISTS idx_verification_identifier
            ON verification (identifier)
    `);
}

async function main() {
    const appEnv = parseAppEnv();
    const databaseUrl = requireEnv("DATABASE_URL");
    const authSecret = requireEnv("AUTH_SECRET");
    const baseUrl = requireEnv("BETTER_AUTH_URL");
    const frontendOrigins = allowedFrontendOrigins(appEnv, process.env.FRONTEND_URL || "");
    const backendUrl = process.env.BACKEND_URL?.trim() || "http://localhost:3000";
    const authRateLimitWindowSecs = parseIntEnv("AUTH_RATE_LIMIT_WINDOW_SECS", 60);
    const authRateLimitMax = parseIntEnv("AUTH_RATE_LIMIT_MAX", 20);
    const authDbMaxConnections = parseIntEnv("AUTH_DB_MAX_CONNECTIONS", 20);
    const port = parseIntEnv("AUTH_PORT", parseIntEnv("PORT", 3001));
    const publicBaseUrl = new URL(baseUrl);
    const cookieDomain = process.env.AUTH_COOKIE_DOMAIN?.trim();

    const pool = new pg.Pool({
        connectionString: databaseUrl,
        max: authDbMaxConnections,
        idleTimeoutMillis: 30_000,
        connectionTimeoutMillis: 5_000,
    });

    const auth = betterAuth({
        database: pool,
        baseURL: baseUrl,
        secret: authSecret,
        useSecureCookies: publicBaseUrl.protocol === "https:",
        ...(cookieDomain
            ? {
                  crossSubDomainCookies: {
                      enabled: true,
                      domain: cookieDomain,
                  },
              }
            : {}),
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
            expiresIn: 60 * 60 * 24 * 7,
            updateAge: 60 * 60 * 24,
        },
        trustedOrigins: [...frontendOrigins, backendUrl],
    });

    const app = new Hono();
    const authWriteRateLimits = new Map<string, { count: number; resetAt: number }>();

    app.use(
        "*",
        cors({
            origin: [...frontendOrigins, backendUrl],
            credentials: true,
        }),
    );

    app.use("/api/auth/*", async (c, next) => {
        if (c.req.method !== "POST") {
            return next();
        }

        const now = Date.now();
        const key = rateLimitKey(c.req.raw);
        const current = authWriteRateLimits.get(key);
        if (!current || current.resetAt <= now) {
            authWriteRateLimits.set(key, {
                count: 1,
                resetAt: now + authRateLimitWindowSecs * 1000,
            });
            return next();
        }

        if (current.count >= authRateLimitMax) {
            return c.json({ error: "Too many auth requests" }, 429);
        }

        current.count += 1;
        authWriteRateLimits.set(key, current);
        return next();
    });

    app.get("/health", async (c) => {
        try {
            await pool.query("SELECT 1");
            await pool.query('SELECT 1 FROM "user" LIMIT 1');
            await pool.query('SELECT 1 FROM "session" LIMIT 1');

            return c.json({ status: "ok", database: "ok" });
        } catch (error) {
            console.error("auth readiness check failed", error);
            return c.json({ status: "degraded", database: "error" }, 503);
        }
    });

    app.on(["GET", "POST"], "/api/auth/**", (c) => {
        return auth.handler(c.req.raw);
    });

    await ensureAuthSchema(pool);

    const server = serve({ fetch: app.fetch, port }, () => {
        console.log(`Auth sidecar running on ${baseUrl}`);
        console.log(`Frontend origins: ${frontendOrigins.join(", ")}`);
        console.log(`Backend origin: ${backendUrl}`);
    });

    const shutdown = async () => {
        console.log("Shutting down auth sidecar...");
        server.close();
        await pool.end();
        process.exit(0);
    };

    process.on("SIGINT", () => {
        void shutdown();
    });

    process.on("SIGTERM", () => {
        void shutdown();
    });
}

void main().catch((error) => {
    console.error("Failed to start auth sidecar", error);
    process.exit(1);
});
