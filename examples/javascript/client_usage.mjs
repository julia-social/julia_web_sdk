import { SignatureClient } from "../../javascript/src/index.js";
// App code should import from the package:
// import { SignatureClient } from "julia_web_sdk";

const client = new SignatureClient({
  baseUrl: process.env.JULIA_WEB_BASE_URL ?? "https://example.com"
});

async function main() {
  const requestId = await client.getSignatureRequestId();
  console.log("request id", requestId);

  const status = await client.getSignatureStatus();
  console.log("signature status", status);

  // The nonce and presentation values are placeholders.
  // Replace with real values produced by your not.bot flow.
  const nonce = "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
  const presentationResponse = await client.generateSignaturePresentation(requestId, nonce);
  console.log("presentation response", presentationResponse);

  await client.verifySignaturePresentation(requestId, []);
  console.log("verification request submitted");
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
