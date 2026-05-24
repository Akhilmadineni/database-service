# database-service

`database-service` is an independent application that provisions and manages per-application PostgreSQL databases and credentials.

## Features
- Creates or reuses one isolated database per app/environment.
- Creates or rotates one dedicated DB role per app/environment.
- Returns application connection details immediately after provisioning.
- Optionally stores credentials in `secret-service`.

## API
- `GET /health`
- `POST /v1/databases/provision`
- `GET /v1/databases/:app/:environment`

Example provision payload:

```json
{
  "app": "clustr",
  "environment": "prod",
  "engine": "postgres",
  "storeCredentials": true
}
```

## Environment
See `.env.example`.

## Run
```bash
npm install
npm run dev
```

## Docker
```bash
docker compose up --build
```

## Notes
- Treat this service as internal-only infrastructure.
- Restrict network access to trusted platform services.
- For production, use a managed or HA PostgreSQL setup for the admin target.

## Roadmap
- See `ROADMAP.md` for production milestones.
