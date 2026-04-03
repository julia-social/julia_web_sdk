# Julia Web SDK (JavaScript)

JavaScript package for integrating Julia web verification flows.

## Install

```bash
cd javascript
npm install
```

## What this package provides

- `SignatureClient`: calls both upstream signature endpoints (`/signature/*`) and SDK auth endpoints (`/auth/*`) from browser or server code.
- `createExpressAuthAdapter`: Express route adapter + websocket proxy.
- `ClaimProperties`: shared claim constants.

## Recommended Integration

- Use `createExpressAuthAdapter` in your site backend as the default integration path.
- Use `SignatureClient` directly for custom/manual endpoint calls or automation scripts.

## Client Methods

- Auth:
  - `getAuthRequestId()`
  - `getAuthStatus()`
  - `generateAuthPresentation(requestId, nonce)`
  - `verifyAuthPresentation(requestId, presentation)`
- Signature:
  - `startSignature(request)`
  - `generatePresentation(request)`
  - `verifyPresentation(request)`

## Express Server Integration

```js
import express from "express";
import session from "express-session";
import http from "node:http";
import {
  ClaimProperties,
  createExpressAuthAdapter,
  createSignatureClient
} from "julia_web_sdk";

const app = express();
app.use(express.json());
app.use(
  session({
    secret: "replace-me",
    resave: false,
    saveUninitialized: false
  })
);

const adapter = createExpressAuthAdapter({
  signatureClient: createSignatureClient(),
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
  }
});

app.use(adapter.router);

const server = http.createServer(app);
adapter.attachWebsocketHandlers(server);
server.listen(3000);
```

## Client Usage

```js
import { SignatureClient } from "julia_web_sdk";

const client = new SignatureClient({ baseUrl: "https://example.com" });
const requestId = await client.getAuthRequestId();
const authenticated = await client.getAuthStatus();
const presentation = await client.generateAuthPresentation(
  requestId,
  "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
);
await client.verifyAuthPresentation(requestId, []);
```

## Examples

- `../examples/javascript/express_server.mjs`
- `../examples/javascript/client_usage.mjs`
