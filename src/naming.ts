import { createHash, randomBytes } from "node:crypto";

const SEGMENT_PATTERN = /^[a-z0-9-]+$/;
const MAX_IDENTIFIER_LENGTH = 63;

function hashSuffix(value: string): string {
  return createHash("sha256").update(value).digest("hex").slice(0, 8);
}

function boundedIdentifier(value: string): string {
  if (value.length <= MAX_IDENTIFIER_LENGTH) return value;
  const suffix = hashSuffix(value);
  return `${value.slice(0, MAX_IDENTIFIER_LENGTH - suffix.length - 1)}_${suffix}`;
}

function normalizeSegment(value: string): string {
  return value.toLowerCase().replace(/-/g, "_");
}

export function assertSafeSegment(value: string, field: string): string {
  const trimmed = value.trim().toLowerCase();
  if (!SEGMENT_PATTERN.test(trimmed)) {
    throw new Error(`Invalid ${field}: use lowercase letters, numbers, and hyphens only`);
  }
  return trimmed;
}

export function buildResourceNames(app: string, environment: string): {
  database: string;
  role: string;
} {
  const appNormalized = normalizeSegment(app);
  const envNormalized = normalizeSegment(environment);

  const database = boundedIdentifier(`app_${appNormalized}_${envNormalized}`);
  const role = boundedIdentifier(`u_${appNormalized}_${envNormalized}`);

  return { database, role };
}

export function generatePassword(length = 48): string {
  const raw = randomBytes(72).toString("base64url");
  return raw.slice(0, length);
}

export function buildConnectionUrl(input: {
  host: string;
  port: number;
  database: string;
  role: string;
  password: string;
  sslMode: string;
}): string {
  const username = encodeURIComponent(input.role);
  const password = encodeURIComponent(input.password);
  const database = encodeURIComponent(input.database);

  return `postgresql://${username}:${password}@${input.host}:${input.port}/${database}?sslmode=${input.sslMode}`;
}
