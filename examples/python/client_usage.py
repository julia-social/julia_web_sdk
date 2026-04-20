import asyncio

from julia_web_sdk import SignatureClient


async def main() -> None:
    client = SignatureClient("https://example.com")
    try:
        request_id = await client.get_signature_request_id()
        print("request id", request_id)

        signature_status = await client.get_signature_status()
        print("signature status", signature_status)

        # Placeholder values for local testing.
        # In production, not.bot provides nonce + presentation values.
        nonce = "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
        presentation = await client.generate_signature_presentation(request_id, nonce)
        print("presentation bytes", len(presentation.compressed_presentation))

        await client.verify_signature_presentation(request_id, [])
        print("verification request submitted")
    finally:
        await client.close()


if __name__ == "__main__":
    asyncio.run(main())
