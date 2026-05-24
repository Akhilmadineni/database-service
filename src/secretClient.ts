export class SecretServiceClient {
  constructor(
    private readonly baseUrl: string,
    private readonly apiKey: string,
    private readonly pathPrefix: string
  ) {}

  async putSecret(input: {
    app: string;
    environment: string;
    key: string;
    value: string;
  }): Promise<void> {
    const namespacedKey = `${this.pathPrefix}.${input.key}`;
    const endpoint = new URL(
      `/v1/apps/${encodeURIComponent(input.app)}/secrets/${encodeURIComponent(namespacedKey)}`,
      this.baseUrl
    );

    const response = await fetch(endpoint, {
      method: "PUT",
      headers: {
        "content-type": "application/json",
        "x-api-key": this.apiKey
      },
      body: JSON.stringify({
        value: input.value,
        environment: input.environment
      })
    });

    if (!response.ok) {
      const details = await response.text();
      throw new Error(
        `Failed to store secret ${namespacedKey} in secret-service (${response.status}): ${details}`
      );
    }
  }
}
