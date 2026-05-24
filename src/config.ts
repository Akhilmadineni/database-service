import { z } from "zod";

const envSchema = z.object({
  PORT: z.coerce.number().int().positive().default(8080),
  API_KEYS: z.string().trim().min(1),
  PG_ADMIN_URL: z.string().url(),
  PG_APP_HOST: z.string().trim().min(1).default("postgres"),
  PG_APP_PORT: z.coerce.number().int().positive().default(5432),
  PG_APP_SSLMODE: z.enum([
    "disable",
    "allow",
    "prefer",
    "require",
    "verify-ca",
    "verify-full"
  ]).default("disable"),
  SECRET_SERVICE_ENABLED: z.string().default("false"),
  SECRET_SERVICE_URL: z.string().url().optional(),
  SECRET_SERVICE_API_KEY: z.string().optional(),
  SECRET_PATH_PREFIX: z.string().trim().min(1).default("database")
});

const parsed = envSchema.parse(process.env);
const secretServiceEnabled = /^(1|true|yes)$/i.test(parsed.SECRET_SERVICE_ENABLED);
const apiKeys = parsed.API_KEYS
  .split(",")
  .map((entry) => entry.trim())
  .filter((entry) => entry.length > 0);

if (apiKeys.length === 0) {
  throw new Error("API_KEYS must include at least one key");
}

if (secretServiceEnabled && (!parsed.SECRET_SERVICE_URL || !parsed.SECRET_SERVICE_API_KEY)) {
  throw new Error(
    "SECRET_SERVICE_URL and SECRET_SERVICE_API_KEY are required when SECRET_SERVICE_ENABLED=true"
  );
}

export const config = {
  port: parsed.PORT,
  apiKeys: new Set(apiKeys),
  postgres: {
    adminUrl: parsed.PG_ADMIN_URL,
    appHost: parsed.PG_APP_HOST,
    appPort: parsed.PG_APP_PORT,
    appSslMode: parsed.PG_APP_SSLMODE
  },
  secretService: {
    enabled: secretServiceEnabled,
    url: parsed.SECRET_SERVICE_URL,
    apiKey: parsed.SECRET_SERVICE_API_KEY,
    pathPrefix: parsed.SECRET_PATH_PREFIX
  }
};
