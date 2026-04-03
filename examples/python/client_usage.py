import asyncio

from julia_web_sdk import SignatureClient


async def main() -> None:
    client = SignatureClient("https://example.com")
    try:
        request_id = await client.get_auth_request_id()
        print("request id", request_id)

        auth_status = await client.get_auth_status()
        print("auth status", auth_status)

        # Placeholder values for local testing.
        # In production, not.bot provides nonce + presentation values.
        nonce = "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
        presentation = await client.generate_auth_presentation(request_id, nonce)
        print("presentation bytes", len(presentation.compressed_presentation))

        await client.verify_auth_presentation(request_id, [])
        print("verification request submitted")
    finally:
        await client.close()


if __name__ == "__main__":
    asyncio.run(main())
