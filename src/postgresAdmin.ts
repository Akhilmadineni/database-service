import { Client } from "pg";

function quoteIdentifier(identifier: string): string {
  return `"${identifier.replaceAll("\"", "\"\"")}"`;
}

function quoteLiteral(value: string): string {
  return `'${value.replaceAll("'", "''")}'`;
}

export class PostgresAdmin {
  constructor(private readonly adminUrl: string) {}

  async provisionDatabase(input: {
    database: string;
    role: string;
    password: string;
  }): Promise<{ databaseCreated: boolean; roleCreated: boolean }> {
    const adminClient = new Client({ connectionString: this.adminUrl });
    await adminClient.connect();

    try {
      const roleExists = await adminClient.query(
        "SELECT 1 FROM pg_roles WHERE rolname = $1",
        [input.role]
      );

      if (roleExists.rowCount === 0) {
        await adminClient.query(
          `CREATE ROLE ${quoteIdentifier(input.role)} LOGIN PASSWORD ${quoteLiteral(input.password)}`
        );
      } else {
        await adminClient.query(
          `ALTER ROLE ${quoteIdentifier(input.role)} WITH LOGIN PASSWORD ${quoteLiteral(input.password)}`
        );
      }

      const databaseExists = await adminClient.query(
        "SELECT 1 FROM pg_database WHERE datname = $1",
        [input.database]
      );

      if (databaseExists.rowCount === 0) {
        await adminClient.query(
          `CREATE DATABASE ${quoteIdentifier(input.database)} OWNER ${quoteIdentifier(input.role)}`
        );
      } else {
        await adminClient.query(
          `ALTER DATABASE ${quoteIdentifier(input.database)} OWNER TO ${quoteIdentifier(input.role)}`
        );
      }

      await this.applyGrants(input.database, input.role);

      return {
        databaseCreated: databaseExists.rowCount === 0,
        roleCreated: roleExists.rowCount === 0
      };
    } finally {
      await adminClient.end();
    }
  }

  async inspect(input: {
    database: string;
    role: string;
  }): Promise<{ databaseExists: boolean; roleExists: boolean }> {
    const adminClient = new Client({ connectionString: this.adminUrl });
    await adminClient.connect();

    try {
      const [databaseQuery, roleQuery] = await Promise.all([
        adminClient.query("SELECT 1 FROM pg_database WHERE datname = $1", [input.database]),
        adminClient.query("SELECT 1 FROM pg_roles WHERE rolname = $1", [input.role])
      ]);

      return {
        databaseExists: (databaseQuery.rowCount ?? 0) > 0,
        roleExists: (roleQuery.rowCount ?? 0) > 0
      };
    } finally {
      await adminClient.end();
    }
  }

  private async applyGrants(database: string, role: string): Promise<void> {
    const databaseUrl = new URL(this.adminUrl);
    databaseUrl.pathname = `/${database}`;

    const appClient = new Client({ connectionString: databaseUrl.toString() });
    await appClient.connect();

    try {
      await appClient.query(`REVOKE ALL ON SCHEMA public FROM PUBLIC`);
      await appClient.query(
        `GRANT USAGE, CREATE ON SCHEMA public TO ${quoteIdentifier(role)}`
      );
      await appClient.query(
        `GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO ${quoteIdentifier(role)}`
      );
      await appClient.query(
        `GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA public TO ${quoteIdentifier(role)}`
      );
      await appClient.query(
        `ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO ${quoteIdentifier(role)}`
      );
      await appClient.query(
        `ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT USAGE, SELECT, UPDATE ON SEQUENCES TO ${quoteIdentifier(role)}`
      );
    } finally {
      await appClient.end();
    }
  }
}
