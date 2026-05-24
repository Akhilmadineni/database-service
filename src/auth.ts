import type { FastifyReply, FastifyRequest } from "fastify";
import { config } from "./config.js";

export function requireApiKey(request: FastifyRequest, reply: FastifyReply): boolean {
  const apiKey = request.headers["x-api-key"];
  const token = Array.isArray(apiKey) ? apiKey[0] : apiKey;

  if (!token || !config.apiKeys.has(token)) {
    reply.code(401).send({ error: "unauthorized" });
    return false;
  }

  return true;
}
