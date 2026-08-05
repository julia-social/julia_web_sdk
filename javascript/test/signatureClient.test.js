import assert from "node:assert/strict";
import test from "node:test";
import { SignatureClient } from "../src/client/signatureClient.js";

test("SignatureClient retains server cookies across requests", async () => {
  const requests = [];
  const fetchImpl = async (_url, options) => {
    requests.push(options);
    return {
      ok: true,
      status: 200,
      text: async () => "true",
      headers: {
        getSetCookie: () => (requests.length === 1 ? ["julia_session=abc; Path=/; HttpOnly"] : [])
      }
    };
  };
  const client = new SignatureClient({ baseUrl: "http://example.test", fetchImpl });

  await client.getSignatureStatus();
  await client.getSignatureStatus();

  assert.equal(requests[1].headers.cookie, "julia_session=abc");
});

test("SignatureClient only sends cookies to matching paths and secure origins", async () => {
  const requests = [];
  const fetchImpl = async (_url, options) => {
    requests.push(options);
    return {
      ok: true,
      status: 200,
      text: async () => "true",
      headers: {
        getSetCookie: () =>
          requests.length === 1 ? ["scoped=value; Path=/signature/notbot; Secure"] : []
      }
    };
  };
  const client = new SignatureClient({ baseUrl: "https://example.test", fetchImpl });

  await client.getSignatureStatus();
  await client.generateSignaturePresentation("request", "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff");

  assert.equal(requests[1].headers.cookie, "scoped=value");
});
