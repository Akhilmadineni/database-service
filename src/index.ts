import Fastify from "fastify";
import { z } from "zod";
import { config } from "./config.js";
import { requireApiKey } from "./auth.js";
import {
  assertSafeSegment,
  buildConnectionUrl,
  buildResourceNames,
  generatePassword
} from "./naming.js";
import { PostgresAdmin } from "./postgresAdmin.js";
import { SecretServiceClient } from "./secretClient.js";

const app = Fastify({ logger: true });
const postgresAdmin = new PostgresAdmin(config.postgres.adminUrl);

const secretClient = config.secretService.enabled
  ? new SecretServiceClient(
    config.secretService.url!,
    config.secretService.apiKey!,
    config.secretService.pathPrefix
  )
  : null;

const provisionSchema = z.object({
  app: z.string().trim().min(2).max(50).regex(/^[a-z0-9-]+$/),
  environment: z.string().trim().min(2).max(50).regex(/^[a-z0-9-]+$/).default("prod"),
  engine: z.literal("postgres").default("postgres"),
  storeCredentials: z.boolean().optional()
});

const statusParamsSchema = z.object({
  app: z.string().trim().min(2).max(50).regex(/^[a-z0-9-]+$/),
  environment: z.string().trim().min(2).max(50).regex(/^[a-z0-9-]+$/)
});

app.get("/health", async () => ({ status: "ok", service: "database-service" }));

app.post("/v1/databases/provision", async (request, reply) => {
  if (!requireApiKey(request, reply)) return;

  const payload = provisionSchema.parse(request.body);
  const appName = assertSafeSegment(payload.app, "app");
  const environment = assertSafeSegment(payload.environment, "environment");

  const names = buildResourceNames(appName, environment);
  const password = generatePassword();
  const connectionUrl = buildConnectionUrl({
    host: config.postgres.appHost,
    port: config.postgres.appPort,
    database: names.database,
    role: names.role,
    password,
    sslMode: config.postgres.appSslMode
  });

  const provisionResult = await postgresAdmin.provisionDatabase({
    database: names.database,
    role: names.role,
    password
  });

  const shouldStoreCredentials = payload.storeCredentials ?? config.secretService.enabled;
  let storedInSecretService = false;

  if (shouldStoreCredentials) {
    if (!secretClient) {
      reply.code(400).send({
        error: "secret_service_not_enabled",
        message: "Set SECRET_SERVICE_ENABLED=true and configure secret-service to store credentials."
      });
      return;
    }

    await Promise.all([
      secretClient.putSecret({
        app: appName,
        environment,
        key: "url",
        value: connectionUrl
      }),
      secretClient.putSecret({
        app: appName,
        environment,
        key: "database",
        value: names.database
      }),
      secretClient.putSecret({
        app: appName,
        environment,
        key: "username",
        value: names.role
      }),
      secretClient.putSecret({
        app: appName,
        environment,
        key: "password",
        value: password
      })
    ]);

    storedInSecretService = true;
  }

  reply.code(200).send({
    app: appName,
    environment,
    engine: payload.engine,
    database: names.database,
    username: names.role,
    password,
    connectionUrl,
    created: {
      database: provisionResult.databaseCreated,
      role: provisionResult.roleCreated
    },
    storedInSecretService
  });
});

app.get("/v1/databases/:app/:environment", async (request, reply) => {
  if (!requireApiKey(request, reply)) return;

  const params = statusParamsSchema.parse(request.params);
  const appName = assertSafeSegment(params.app, "app");
  const environment = assertSafeSegment(params.environment, "environment");
  const names = buildResourceNames(appName, environment);

  const status = await postgresAdmin.inspect({
    database: names.database,
    role: names.role
  });

  reply.code(200).send({
    app: appName,
    environment,
    engine: "postgres",
    database: names.database,
    username: names.role,
    ...status
  });
});

app.setErrorHandler((error, _request, reply) => {
  if (error instanceof z.ZodError) {
    reply.code(400).send({
      error: "validation_error",
      details: error.flatten()
    });
    return;
  }

  reply.code(500).send({
    error: "internal_error",
    message: error instanceof Error ? error.message : "Unknown error"
  });
});

app.listen({ host: "0.0.0.0", port: config.port })
  .then(() => app.log.info(`database-service listening on ${config.port}`))
  .catch((error) => {
    app.log.error(error);
    process.exit(1);
  });
