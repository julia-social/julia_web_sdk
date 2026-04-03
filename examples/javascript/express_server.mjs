import express from "express";
import session from "express-session";
import http from "node:http";
import {
  ClaimProperties,
  createExpressAuthAdapter,
  createSignatureClient
} from "../../javascript/src/index.js";

const app = express();
app.use(express.json());
app.use(
  session({
    secret: process.env.SESSION_SECRET ?? "change-me",
    resave: false,
    saveUninitialized: false
  })
);

const signatureClient = createSignatureClient();
const authAdapter = createExpressAuthAdapter({
  signatureClient,
  requestedClaims: [
    ClaimProperties.Notbot0,
    ClaimProperties.SitePass,
    ClaimProperties.FirstName,
    ClaimProperties.AgeOver18
  ],
  requireSitePass: true,
  messageGenerator: () => "Verifying My Identity with example.com",
  onSuccess: async (verifyResponse, sessionData) => {
    sessionData.juliaSignatureVerification = verifyResponse;
  },
  onFailure: async (error) => {
    console.error("verification error", error);
  }
});

app.use(authAdapter.router);
app.get("/health", (_req, res) => res.json({ ok: true }));

const server = http.createServer(app);
authAdapter.attachWebsocketHandlers(server);

const port = Number(process.env.PORT ?? 3000);
server.listen(port, () => {
  console.log(`server listening on port ${port}`);
});
