import express from "express";
import { WebSocket, WebSocketServer } from "ws";
import { SignatureClient } from "../client/signatureClient.js";

function toWsUrl(baseUrl, path) {
  const normalized = baseUrl.replace(/^http/, "ws").replace(/\/$/, "");
  return `${normalized}${path}`;
}

function safeSessionSave(session) {
  if (!session || typeof session.save !== "function") {
    return Promise.resolve();
  }
  return new Promise((resolve, reject) => {
    session.save((error) => (error ? reject(error) : resolve()));
  });
}

function loadSessionById(store, sessionId) {
  if (!store || typeof store.get !== "function" || !sessionId) {
    return Promise.resolve(null);
  }
  return new Promise((resolve, reject) => {
    store.get(sessionId, (error, value) => {
      if (error) {
        reject(error);
        return;
      }
      resolve(value ?? null);
    });
  });
}

export function createExpressAuthAdapter({
  signatureClient = new SignatureClient(),
  requestedClaims = [],
  requireSitePass = false,
  messageGenerator = () => "",
  onSuccess = async (verifyResponse, session) => {
    session.juliaSignatureVerification = verifyResponse;
  },
  onFailure = async () => {},
  expireTimeSeconds = 3600
} = {}) {
  const router = express.Router();
  const requestToSession = new Map();

  router.get("/auth/notbot", async (req, res) => {
    try {
      delete req.session?.juliaSignatureVerification;
      const response = await signatureClient.startSignature({
        requested_credentials: [...requestedClaims],
        require_site_pass: requireSitePass,
        required_alias_launcher: null,
        requested_message: Array.from(Buffer.from(messageGenerator(), "utf8")),
        expires: Math.floor(Date.now() / 1000) + expireTimeSeconds
      });
      if (response?.request_id && req.sessionID) {
        requestToSession.set(response.request_id, req.sessionID);
      }
      res.json(response?.request_id ?? response);
    } catch (error) {
      await onFailure(error);
      res.status(502).json({ error: String(error) });
    }
  });

  router.get("/auth/status", async (req, res) => {
    const ok = Boolean(req.session?.juliaSignatureVerification);
    res.json(ok);
  });

  router.post("/auth/notbot/:requestId", async (req, res) => {
    const payload = req.body;
    if (!payload || typeof payload !== "object" || typeof payload.nonce !== "string") {
      res.status(400).json({ error: "Missing payload.nonce" });
      return;
    }

    try {
      const response = await signatureClient.generatePresentation({
        request_id: req.params.requestId,
        nonce: payload.nonce
      });
      res.json({ compressed_presentation: response.compressed_presentation });
    } catch (error) {
      await onFailure(error);
      res.status(502).json({ error: String(error) });
    }
  });

  router.post("/auth/verify/:requestId", async (req, res) => {
    const payload = req.body;
    if (!payload || typeof payload !== "object" || !Array.isArray(payload.presentation)) {
      res.status(400).json({ error: "Missing payload.presentation" });
      return;
    }

    let verifyResponse;
    try {
      verifyResponse = await signatureClient.verifyPresentation({
        request_id: req.params.requestId,
        presentation: payload.presentation
      });
    } catch (error) {
      await onFailure(error);
      res.status(422).json({ error: String(error) });
      return;
    }

    try {
      const sessionId = requestToSession.get(req.params.requestId);
      requestToSession.delete(req.params.requestId);
      const targetSession =
        sessionId === req.sessionID
          ? req.session
          : (await loadSessionById(req.sessionStore, sessionId)) ?? req.session;

      await onSuccess(verifyResponse, targetSession);
      await safeSessionSave(targetSession);
      res.status(204).send();
    } catch (error) {
      await onFailure(error);
      res.status(500).json({ error: String(error) });
    }
  });

  function attachWebsocketHandlers(httpServer) {
    const wsServer = new WebSocketServer({ noServer: true });

    httpServer.on("upgrade", (request, socket, head) => {
      const targetPath = new URL(request.url, "http://localhost").pathname;
      if (targetPath !== "/auth/honestbot" && targetPath !== "/calculate_site_pass") {
        return;
      }

      wsServer.handleUpgrade(request, socket, head, (clientSocket) => {
        const upstreamPath = targetPath === "/auth/honestbot" ? "/signature/honestbot" : "/calculate_site_pass";
        const upstreamHeaders = {};
        if (targetPath === "/auth/honestbot" && request.headers["x-presentation-hash"]) {
          upstreamHeaders["x-presentation-hash"] = request.headers["x-presentation-hash"];
        }
        if (targetPath === "/calculate_site_pass" && request.headers["x-site-pass"]) {
          upstreamHeaders["x-site-pass"] = request.headers["x-site-pass"];
        }

        const upstreamSocket = new WebSocket(toWsUrl(signatureClient.baseUrl, upstreamPath), {
          headers: upstreamHeaders
        });

        const closeBoth = () => {
          if (clientSocket.readyState === WebSocket.OPEN) {
            clientSocket.close();
          }
          if (upstreamSocket.readyState === WebSocket.OPEN) {
            upstreamSocket.close();
          }
        };

        clientSocket.on("message", (message, isBinary) => {
          if (upstreamSocket.readyState === WebSocket.OPEN) {
            upstreamSocket.send(message, { binary: isBinary });
          }
        });

        upstreamSocket.on("message", (message, isBinary) => {
          if (clientSocket.readyState === WebSocket.OPEN) {
            clientSocket.send(message, { binary: isBinary });
          }
        });

        clientSocket.on("close", closeBoth);
        upstreamSocket.on("close", closeBoth);
        clientSocket.on("error", closeBoth);
        upstreamSocket.on("error", closeBoth);
      });
    });
  }

  return {
    router,
    attachWebsocketHandlers
  };
}
