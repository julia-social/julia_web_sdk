import * as QRCode from "qrcode";
import { byteArray, bytes32 } from "../models.js";

function trimTrailingSlash(input) {
  return input.endsWith("/") ? input.slice(0, -1) : input;
}

function isBrowserRuntime() {
  return typeof window !== "undefined" && typeof window.document !== "undefined";
}

function splitSetCookieHeader(header) {
  // A comma is part of an Expires attribute, but otherwise begins the next cookie.
  return header.split(/,(?=\s*[^;=]+=[^;]*)/);
}

function getSetCookieHeaders(headers) {
  if (typeof headers?.getSetCookie === "function") {
    return headers.getSetCookie();
  }
  const header = headers?.get?.("set-cookie");
  return header ? splitSetCookieHeader(header) : [];
}

class CookieJar {
  #cookies = [];

  store(url, setCookieHeaders) {
    for (const header of setCookieHeaders) {
      const parts = header.split(";").map((part) => part.trim());
      const [name, value] = parts.shift().split(/=(.*)/s);
      if (!name) continue;

      const cookie = {
        name,
        value: value ?? "",
        domain: url.hostname.toLowerCase(),
        hostOnly: true,
        path: this.#defaultPath(url.pathname),
        secure: false,
        expiresAt: null
      };
      let validDomain = true;

      for (const attribute of parts) {
        const [rawName, rawValue] = attribute.split(/=(.*)/s);
        const attributeName = rawName.toLowerCase();
        if (attributeName === "domain" && rawValue) {
          const domain = rawValue.replace(/^\./, "").toLowerCase();
          if (url.hostname !== domain && !url.hostname.endsWith(`.${domain}`)) {
            validDomain = false;
            break;
          }
          cookie.domain = domain;
          cookie.hostOnly = false;
        } else if (attributeName === "path" && rawValue?.startsWith("/")) {
          cookie.path = rawValue;
        } else if (attributeName === "secure") {
          cookie.secure = true;
        } else if (attributeName === "max-age") {
          const seconds = Number(rawValue);
          if (Number.isFinite(seconds)) cookie.expiresAt = Date.now() + seconds * 1000;
        } else if (attributeName === "expires") {
          const expiresAt = Date.parse(rawValue);
          if (!Number.isNaN(expiresAt)) cookie.expiresAt = expiresAt;
        }
      }
      if (!validDomain) continue;

      this.#cookies = this.#cookies.filter(
        (existing) =>
          existing.name !== cookie.name ||
          existing.domain !== cookie.domain ||
          existing.path !== cookie.path
      );
      if (cookie.expiresAt === null || cookie.expiresAt > Date.now()) {
        this.#cookies.push(cookie);
      }
    }
  }

  header(url) {
    const now = Date.now();
    this.#cookies = this.#cookies.filter((cookie) => cookie.expiresAt === null || cookie.expiresAt > now);
    return this.#cookies
      .filter((cookie) => this.#matches(cookie, url))
      .sort((left, right) => right.path.length - left.path.length)
      .map((cookie) => `${cookie.name}=${cookie.value}`)
      .join("; ");
  }

  #defaultPath(pathname) {
    if (!pathname || pathname === "/") return "/";
    const lastSlash = pathname.lastIndexOf("/");
    return lastSlash === 0 ? "/" : pathname.slice(0, lastSlash);
  }

  #matches(cookie, url) {
    const host = url.hostname.toLowerCase();
    const domainMatches = cookie.hostOnly
      ? host === cookie.domain
      : host === cookie.domain || host.endsWith(`.${cookie.domain}`);
    const pathMatches =
      url.pathname === cookie.path ||
      (url.pathname.startsWith(cookie.path) &&
        (cookie.path.endsWith("/") || url.pathname[cookie.path.length] === "/"));
    return domainMatches && pathMatches && (!cookie.secure || url.protocol === "https:");
  }
}

function parseRequestId(payload) {
  if (typeof payload === "string") {
    return payload;
  }
  if (payload && typeof payload === "object") {
    if (typeof payload.request_id === "string") {
      return payload.request_id;
    }
    if (typeof payload.requestId === "string") {
      return payload.requestId;
    }
  }
  return null;
}

function resolveQrEncoder() {
  const toDataURL = QRCode?.toDataURL ?? QRCode?.default?.toDataURL;
  if (typeof toDataURL !== "function") {
    throw new Error("qrcode.toDataURL is not available");
  }
  return toDataURL;
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
    this.fetchImpl = typeof fetchImpl.bind === "function" ? fetchImpl.bind(globalThis) : fetchImpl;
    this.cookieJar = isBrowserRuntime() ? null : new CookieJar();
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

  async getSignatureRequestId() {
    const payload = await this.#requestSdkSignature("/signature/notbot", { method: "GET" });
    const requestId = parseRequestId(payload);
    if (typeof requestId !== "string") {
      throw new JuliaWebSdkError("Missing request id in /signature/notbot response", null, payload);
    }
    const url = this.#buildNotbotUrl(requestId);
    const toDataURL = resolveQrEncoder();
    const qr_code = await toDataURL(url, {
      errorCorrectionLevel: "M",
      margin: 2,
      scale: 8
    });
    return { request_id: requestId, qr_code, url };
  }

  async getSignatureStatus() {
    const payload = await this.#requestSdkSignature("/signature/status", { method: "GET" });
    if (typeof payload === "boolean") {
      return payload;
    }
    throw new JuliaWebSdkError("Invalid /signature/status response type", null, payload);
  }

  async generateSignaturePresentation(requestId, nonce) {
    bytes32(nonce);
    return this.#requestSdkSignature(`/signature/notbot/${encodeURIComponent(requestId)}`, {
      method: "POST",
      body: { nonce }
    });
  }

  async verifySignaturePresentation(requestId, presentation) {
    byteArray(presentation);
    await this.#requestSdkSignature(`/signature/verify/${encodeURIComponent(requestId)}`, {
      method: "POST",
      body: { presentation }
    });
  }

  #buildNotbotUrl(requestId) {
    const parsed = this.#parseBaseUrl();
    const defaultPort = parsed.protocol === "https:" ? 443 : 80;
    const port = parsed.port ? Number(parsed.port) : defaultPort;
    return `https://not.bot/s1/${requestId}/${parsed.hostname}/${port}`;
  }

  #parseBaseUrl() {
    try {
      return new URL(this.baseUrl);
    } catch (_error) {
      if (typeof window !== "undefined" && window.location?.origin) {
        return new URL(this.baseUrl, window.location.origin);
      }
      return new URL("http://localhost:80");
    }
  }

  #requestUrl(path) {
    if (isBrowserRuntime()) {
      return new URL(`${this.baseUrl}${path}`, window.location.origin);
    }
    return new URL(`${this.baseUrl}${path}`);
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

  async #requestSdkSignature(path, { method, body = undefined }) {
    return this.#request(path, {
      method,
      body,
      headers: {},
      serviceName: "Signature",
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
      const requestUrl = this.#requestUrl(path);
      const requestHeaders = {
        "content-type": "application/json",
        ...headers
      };
      const cookieHeader = this.cookieJar?.header(requestUrl);
      if (cookieHeader) requestHeaders.cookie = cookieHeader;

      response = await this.fetchImpl(`${this.baseUrl}${path}`, {
        method,
        headers: requestHeaders,
        body: body == null ? undefined : JSON.stringify(body),
        credentials: includeCredentials ? "include" : undefined,
        signal: controller.signal
      });
      this.cookieJar?.store(requestUrl, getSetCookieHeaders(response.headers));
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
