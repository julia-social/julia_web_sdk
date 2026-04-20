package examples.java;

import social.julia.sdk.client.SignatureClient;

import java.util.List;

public final class SignatureClientExample {
    private SignatureClientExample() {
    }

    public static void main(String[] args) {
        SignatureClient client = new SignatureClient("https://example.com");

        String requestId = client.getSignatureRequestId().join();
        System.out.println("request id: " + requestId);

        Boolean status = client.getSignatureStatus().join();
        System.out.println("signature status: " + status);

        // Placeholder values for local testing.
        // In production, not.bot provides nonce + presentation values.
        String nonce = "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        var presentation = client.generateSignaturePresentation(requestId, nonce).join();
        System.out.println("presentation bytes: " + presentation.compressedPresentation.size());

        client.verifySignaturePresentation(requestId, List.of()).join();
        System.out.println("verification request submitted");
    }
}
