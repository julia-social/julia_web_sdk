# Julia Web SDK (Java)

Java package for Julia web verification flows.

## Build

```bash
cd java
mvn -q package
```

## What this package provides

- `social.julia.sdk.client.SignatureClient`: async client for both upstream and SDK `/signature/*` endpoints.
- `social.julia.sdk.server.SpringSignatureController`: Spring MVC signature endpoint adapter.
- `social.julia.sdk.server.JuliaWebSocketConfig`: Spring websocket proxy registration.
- `social.julia.sdk.ClaimProperties`: claim constants.

## Recommended Integration

- Use `SpringSignatureController` (with `SignatureAdapterConfig`) in your backend as the default integration path.
- Use `SignatureClient` directly for custom/manual endpoint calls or automation scripts.

## Client Methods

- Signature SDK:
  - `getSignatureRequestId()`
  - `getSignatureStatus()`
  - `generateSignaturePresentation(requestId, nonce)`
  - `verifySignaturePresentation(requestId, presentation)`
- Signature:
  - `startSignature(StartSignatureRequest)`
  - `generatePresentation(GeneratePresentationRequest)`
  - `verifyPresentation(VerifySignatureRequest)`

## Spring Server Integration

```java
@Configuration
public class SdkConfig {
    @Bean
    public SignatureClient signatureClient() {
        return SignatureClient.fromEnv();
    }

    @Bean
    public SignatureAdapterConfig signatureAdapterConfig() {
        return SignatureAdapterConfig.builder()
                .requestedClaims(List.of(
                        ClaimProperties.NOTBOT_0,
                        ClaimProperties.SITE_PASS,
                        ClaimProperties.FIRST_NAME,
                        ClaimProperties.AGE_OVER_18
                ))
                .requireSitePass(true)
                .messageGenerator(() -> "Verifying My Identity with example.com")
                .build();
    }

    @Bean
    public SpringSignatureController springSignatureController(SignatureClient client, SignatureAdapterConfig config) {
        return new SpringSignatureController(client, config);
    }
}
```

## Client Usage

```java
SignatureClient client = new SignatureClient("https://example.com");
String requestId = client.getSignatureRequestId().join();
boolean status = client.getSignatureStatus().join();
client.generateSignaturePresentation(
        requestId,
        "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
).join();
client.verifySignaturePresentation(requestId, List.of()).join();
```

## Examples

- `../examples/java/SpringSdkConfig.java`
- `../examples/java/SignatureClientExample.java`
