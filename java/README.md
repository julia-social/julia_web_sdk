# Julia Web SDK (Java)

Java package for Julia web verification flows.

## Build

```bash
cd java
mvn -q package
```

## What this package provides

- `social.julia.sdk.client.SignatureClient`: async client for both `/signature/*` and `/auth/*`.
- `social.julia.sdk.server.SpringAuthController`: Spring MVC auth endpoint adapter.
- `social.julia.sdk.server.JuliaWebSocketConfig`: Spring websocket proxy registration.
- `social.julia.sdk.ClaimProperties`: claim constants.

## Recommended Integration

- Use `SpringAuthController` (with `AuthAdapterConfig`) in your backend as the default integration path.
- Use `SignatureClient` directly for custom/manual endpoint calls or automation scripts.

## Client Methods

- Auth:
  - `getAuthRequestId()`
  - `getAuthStatus()`
  - `generateAuthPresentation(requestId, nonce)`
  - `verifyAuthPresentation(requestId, presentation)`
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
    public AuthAdapterConfig authAdapterConfig() {
        return AuthAdapterConfig.builder()
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
    public SpringAuthController springAuthController(SignatureClient client, AuthAdapterConfig config) {
        return new SpringAuthController(client, config);
    }
}
```

## Client Usage

```java
SignatureClient client = new SignatureClient("https://example.com");
String requestId = client.getAuthRequestId().join();
boolean status = client.getAuthStatus().join();
client.generateAuthPresentation(
        requestId,
        "0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
).join();
client.verifyAuthPresentation(requestId, List.of()).join();
```

## Examples

- `../examples/java/SpringSdkConfig.java`
- `../examples/java/SignatureClientExample.java`
