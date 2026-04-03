import { byteArray, bytes32 } from "../models.js";

function trimTrailingSlash(input) {
  return input.endsWith("/") ? input.slice(0, -1) : input;
}

export class JuliaWebSdkError extends Error {
  constructor(message, status = null, body = null) {
    super(message);
    this.name = "JuliaWebSdkError";
    this.status = status;
    this.body = body;
  }
}

export class SignatureClient {
  constructor({
    baseUrl = null,
    host = process.env.SIGNATURE_HOSTNAME ?? "localhost",
    port = Number(process.env.SIGNATURE_PORT ?? 8080),
    apiKey = process.env.SIGNATURE_API_KEY ?? "CHANGE_ME",
    secure = host !== "localhost",
    timeoutMs = 180_000,
    fetchImpl = globalThis.fetch
  } = {}) {
    if (!fetchImpl) {
      throw new Error("No fetch implementation available");
    }
    const resolvedBaseUrl = baseUrl ?? `${secure ? "https" : "http"}://${host}:${port}`;
    this.baseUrl = trimTrailingSlash(resolvedBaseUrl);
    this.apiKey = apiKey;
    this.timeoutMs = timeoutMs;
    this.fetchImpl = fetchImpl;
  }

  async startSignature(request) {
    return this.#requestSignature("/signature/start", request);
  }

  async generatePresentation(request) {
    bytes32(request?.nonce);
    return this.#requestSignature("/signature/presentation", request);
  }

  async verifyPresentation(request) {
    byteArray(request?.presentation ?? []);
    return this.#requestSignature("/signature/verify", request);
  }

  async getAuthRequestId() {
    const payload = await this.#requestAuth("/auth/notbot", { method: "GET" });
    if (typeof payload === "string") {
      return payload;
    }
    throw new JuliaWebSdkError("Missing request id in /auth/notbot response", null, payload);
  }

  async getAuthStatus() {
    const payload = await this.#requestAuth("/auth/status", { method: "GET" });
    if (typeof payload === "boolean") {
      return payload;
    }
    throw new JuliaWebSdkError("Invalid /auth/status response type", null, payload);
  }

  async generateAuthPresentation(requestId, nonce) {
    bytes32(nonce);
    return this.#requestAuth(`/auth/notbot/${encodeURIComponent(requestId)}`, {
      method: "POST",
      body: { nonce }
    });
  }

  async verifyAuthPresentation(requestId, presentation) {
    byteArray(presentation);
    await this.#requestAuth(`/auth/verify/${encodeURIComponent(requestId)}`, {
      method: "POST",
      body: { presentation }
    });
  }

  async #requestSignature(path, payload) {
    if (!this.apiKey) {
      throw new JuliaWebSdkError("apiKey is required for signature endpoints");
    }
    return this.#request(path, {
      method: "POST",
      body: payload,
      headers: { "api-key": this.apiKey },
      serviceName: "Signature",
      includeCredentials: false
    });
  }

  async #requestAuth(path, { method, body = undefined }) {
    return this.#request(path, {
      method,
      body,
      headers: {},
      serviceName: "Auth",
      includeCredentials: true
    });
  }

  async #request(
    path,
    { method, body = undefined, headers = {}, serviceName, includeCredentials }
  ) {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), this.timeoutMs);
    let response;

    try {
      response = await this.fetchImpl(`${this.baseUrl}${path}`, {
        method,
        headers: {
          "content-type": "application/json",
          ...headers
        },
        body: body == null ? undefined : JSON.stringify(body),
        credentials: includeCredentials ? "include" : undefined,
        signal: controller.signal
      });
    } catch (error) {
      clearTimeout(timeout);
      throw new JuliaWebSdkError(
        `Error connecting to ${serviceName.toLowerCase()} service: ${String(error)}`
      );
    }

    clearTimeout(timeout);
    const rawBody = await response.text();
    let parsed = null;
    if (rawBody.length > 0) {
      try {
        parsed = JSON.parse(rawBody);
      } catch (error) {
        throw new JuliaWebSdkError(
          `Error parsing ${serviceName.toLowerCase()} response JSON: ${String(error)} - ${rawBody}`,
          response.status,
          rawBody
        );
      }
    }

    if (!response.ok) {
      throw new JuliaWebSdkError(
        `${serviceName} service error ${response.status}: ${rawBody}`,
        response.status,
        parsed ?? rawBody
      );
    }

    return parsed;
  }
}

export function createSignatureClient(options = {}) {
  return new SignatureClient(options);
}
