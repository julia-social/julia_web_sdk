package social.julia.sdk.client;

import com.fasterxml.jackson.databind.ObjectMapper;
import social.julia.sdk.JuliaWebSdkException;
import social.julia.sdk.model.SignatureModels.ClientPresentation;
import social.julia.sdk.model.SignatureModels.GeneratePresentationRequest;
import social.julia.sdk.model.SignatureModels.GeneratePresentationResponse;
import social.julia.sdk.model.SignatureModels.SignatureRequest;
import social.julia.sdk.model.SignatureModels.StartSignatureRequest;
import social.julia.sdk.model.SignatureModels.StartSignatureResponse;
import social.julia.sdk.model.SignatureModels.VerifySignatureRequest;
import social.julia.sdk.model.SignatureModels.VerifySignatureResponse;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.time.Duration;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.List;

public class SignatureClient {
    private final HttpClient httpClient;
    private final ObjectMapper objectMapper;
    private final String baseUrl;
    private final String apiKey;

    public SignatureClient(String baseUrl) {
        this(
                baseUrl,
                System.getenv().getOrDefault("SIGNATURE_API_KEY", "CHANGE_ME"),
                HttpClient.newBuilder().connectTimeout(Duration.ofSeconds(180)).build(),
                new ObjectMapper()
        );
    }

    public SignatureClient(String baseUrl, String apiKey) {
        this(baseUrl, apiKey, HttpClient.newBuilder()
                .connectTimeout(Duration.ofSeconds(180))
                .build(), new ObjectMapper());
    }

    public SignatureClient(String baseUrl, String apiKey, HttpClient httpClient, ObjectMapper objectMapper) {
        this.baseUrl = trimTrailingSlash(baseUrl);
        this.apiKey = apiKey;
        this.httpClient = httpClient;
        this.objectMapper = objectMapper;
    }

    public static SignatureClient fromEnv() {
        String host = System.getenv().getOrDefault("SIGNATURE_HOSTNAME", "localhost");
        int port = Integer.parseInt(System.getenv().getOrDefault("SIGNATURE_PORT", "8080"));
        String key = System.getenv().getOrDefault("SIGNATURE_API_KEY", "CHANGE_ME");
        String scheme = host.equals("localhost") ? "http" : "https";
        return new SignatureClient(String.format("%s://%s:%d", scheme, host, port), key);
    }

    public String getBaseUrl() {
        return baseUrl;
    }

    public CompletableFuture<StartSignatureResponse> startSignature(StartSignatureRequest request) {
        return postSignature("/signature/start", request, StartSignatureResponse.class);
    }

    public CompletableFuture<GeneratePresentationResponse> generatePresentation(GeneratePresentationRequest request) {
        return postSignature("/signature/presentation", request, GeneratePresentationResponse.class);
    }

    public CompletableFuture<VerifySignatureResponse> verifyPresentation(VerifySignatureRequest request) {
        return postSignature("/signature/verify", request, VerifySignatureResponse.class);
    }

    public CompletableFuture<String> getAuthRequestId() {
        HttpRequest request = HttpRequest.newBuilder(URI.create(baseUrl + "/auth/notbot"))
                .GET()
                .build();

        return httpClient.sendAsync(request, HttpResponse.BodyHandlers.ofString())
                .thenApply(response -> parseAuthResponse(response, String.class));
    }

    public CompletableFuture<Boolean> getAuthStatus() {
        HttpRequest request = HttpRequest.newBuilder(URI.create(baseUrl + "/auth/status"))
                .GET()
                .build();

        return httpClient.sendAsync(request, HttpResponse.BodyHandlers.ofString())
                .thenApply(response -> parseAuthResponse(response, Boolean.class));
    }

    public CompletableFuture<GeneratePresentationResponse> generateAuthPresentation(String requestId, String nonce) {
        SignatureRequest payload = new SignatureRequest();
        payload.nonce = nonce;
        return postAuth("/auth/notbot/" + requestId, payload, GeneratePresentationResponse.class);
    }

    public CompletableFuture<Void> verifyAuthPresentation(String requestId, List<Integer> presentation) {
        ClientPresentation payload = new ClientPresentation();
        payload.presentation = presentation;
        return postAuth("/auth/verify/" + requestId, payload, Void.class)
                .thenApply(_unused -> null);
    }

    private <T> CompletableFuture<T> postSignature(String path, Object payload, Class<T> responseType) {
        final String body;
        try {
            body = objectMapper.writeValueAsString(payload);
        } catch (Exception e) {
            throw new JuliaWebSdkException("Failed to serialize request body", e);
        }

        HttpRequest request = HttpRequest.newBuilder(URI.create(baseUrl + path))
                .timeout(Duration.ofSeconds(180))
                .header("content-type", "application/json")
                .header("api-key", apiKey)
                .POST(HttpRequest.BodyPublishers.ofString(body))
                .build();

        return httpClient.sendAsync(request, HttpResponse.BodyHandlers.ofString())
                .thenApply(response -> parseSignatureResponse(response, responseType));
    }

    private <T> CompletableFuture<T> postAuth(String path, Object payload, Class<T> responseType) {
        final String body;
        try {
            body = objectMapper.writeValueAsString(payload);
        } catch (Exception e) {
            throw new JuliaWebSdkException("Failed to serialize request body", e);
        }

        HttpRequest request = HttpRequest.newBuilder(URI.create(baseUrl + path))
                .timeout(Duration.ofSeconds(30))
                .header("content-type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString(body))
                .build();

        return httpClient.sendAsync(request, HttpResponse.BodyHandlers.ofString())
                .thenApply(response -> parseAuthResponse(response, responseType));
    }

    private <T> T parseSignatureResponse(HttpResponse<String> response, Class<T> responseType) {
        String body = response.body() == null ? "" : response.body();
        if (response.statusCode() < 200 || response.statusCode() >= 300) {
            throw new CompletionException(
                    new JuliaWebSdkException(
                            "Signature service error " + response.statusCode() + ": " + body,
                            response.statusCode(),
                            body
                    )
            );
        }

        try {
            return objectMapper.readValue(body, responseType);
        } catch (Exception e) {
            throw new CompletionException(
                    new JuliaWebSdkException("Failed to parse signature response: " + body, e)
            );
        }
    }

    private <T> T parseAuthResponse(HttpResponse<String> response, Class<T> responseType) {
        String responseBody = response.body() == null ? "" : response.body();
        if (response.statusCode() < 200 || response.statusCode() >= 300) {
            throw new CompletionException(new JuliaWebSdkException(
                    "Auth service error " + response.statusCode() + ": " + responseBody,
                    response.statusCode(),
                    responseBody
            ));
        }
        if (responseType == Void.class) {
            return null;
        }
        if (responseBody.isEmpty()) {
            throw new CompletionException(new JuliaWebSdkException("Empty auth response body"));
        }
        try {
            return objectMapper.readValue(responseBody, responseType);
        } catch (Exception e) {
            throw new CompletionException(new JuliaWebSdkException("Failed to parse auth response", e));
        }
    }

    private static String trimTrailingSlash(String value) {
        if (value.endsWith("/")) {
            return value.substring(0, value.length() - 1);
        }
        return value;
    }
}
