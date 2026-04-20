package social.julia.sdk.server;

import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpSession;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RestController;
import social.julia.sdk.client.SignatureClient;
import social.julia.sdk.model.SignatureModels.ClientPresentation;
import social.julia.sdk.model.SignatureModels.GeneratePresentationRequest;
import social.julia.sdk.model.SignatureModels.ServerPresentation;
import social.julia.sdk.model.SignatureModels.SignatureRequest;
import social.julia.sdk.model.SignatureModels.StartSignatureRequest;
import social.julia.sdk.model.SignatureModels.VerifySignatureRequest;
import social.julia.sdk.model.SignatureModels.VerifySignatureResponse;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletionException;
import java.util.concurrent.ConcurrentHashMap;

@RestController
public class SpringSignatureController {
    private final SignatureClient signatureClient;
    private final SignatureAdapterConfig config;
    private final Map<String, String> sessionSignatures = new ConcurrentHashMap<>();

    public SpringSignatureController(SignatureClient signatureClient, SignatureAdapterConfig config) {
        this.signatureClient = signatureClient;
        this.config = config;
    }

    @GetMapping("/signature/notbot")
    public ResponseEntity<?> getSignatureUrl(HttpServletRequest request) {
        HttpSession session = request.getSession(false);
        if (session != null) {
            session.removeAttribute(config.getSessionAttributeName());
        }

        StartSignatureRequest payload = new StartSignatureRequest();
        payload.requestedCredentials = new ArrayList<>(config.getRequestedClaims());
        payload.requireSitePass = config.isRequireSitePass();
        payload.requiredAliasLauncher = null;
        payload.requestedMessage = utf8ToByteList(config.getMessageGenerator().get());
        payload.expires = (System.currentTimeMillis() / 1000L) + config.getExpireTimeSeconds();

        try {
            var response = signatureClient.startSignature(payload).join();
            String sessionId = request.getRequestedSessionId();
            if (sessionId == null) {
                HttpSession created = request.getSession(false);
                if (created != null) {
                    sessionId = created.getId();
                }
            }
            sessionSignatures.put(response.requestId, sessionId);
            return ResponseEntity.ok(response.requestId);
        } catch (Exception error) {
            handleFailure(error);
            return ResponseEntity.status(HttpStatus.BAD_GATEWAY)
                    .body(Map.of("error", rootCause(error).toString()));
        }
    }

    @GetMapping("/signature/status")
    public ResponseEntity<Boolean> getSignatureStatus(HttpServletRequest request) {
        HttpSession session = request.getSession(false);
        boolean status = session != null && session.getAttribute(config.getSessionAttributeName()) != null;
        return ResponseEntity.ok(status);
    }

    @PostMapping("/signature/notbot/{requestId}")
    public ResponseEntity<?> getRequestPresentation(
            @PathVariable("requestId") String requestId,
            @RequestBody SignatureRequest payload
    ) {
        GeneratePresentationRequest request = new GeneratePresentationRequest();
        request.requestId = requestId;
        request.nonce = payload.nonce;

        try {
            var response = signatureClient.generatePresentation(request).join();
            ServerPresentation body = new ServerPresentation();
            body.compressedPresentation = response.compressedPresentation;
            return ResponseEntity.ok(body);
        } catch (Exception error) {
            handleFailure(error);
            return ResponseEntity.status(HttpStatus.BAD_GATEWAY)
                    .body(Map.of("error", rootCause(error).toString()));
        }
    }

    @PostMapping("/signature/verify/{requestId}")
    public ResponseEntity<?> verifyPresentation(
            @PathVariable("requestId") String requestId,
            @RequestBody ClientPresentation payload,
            HttpServletRequest servletRequest
    ) {
        VerifySignatureRequest request = new VerifySignatureRequest();
        request.requestId = requestId;
        request.presentation = payload.presentation;

        final VerifySignatureResponse verifyResponse;
        try {
            verifyResponse = signatureClient.verifyPresentation(request).join();
        } catch (Exception error) {
            handleFailure(error);
            return ResponseEntity.status(HttpStatus.UNPROCESSABLE_ENTITY)
                    .body(Map.of("error", rootCause(error).toString()));
        }

        try {
            String sessionId = sessionSignatures.remove(requestId);
            HttpSession session = config.getSessionResolver().resolve(sessionId, servletRequest);
            if (session == null) {
                session = servletRequest.getSession(true);
            }
            config.getOnSuccess().handle(verifyResponse, session);
            return ResponseEntity.noContent().build();
        } catch (Exception error) {
            handleFailure(error);
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR)
                    .body(Map.of("error", rootCause(error).toString()));
        }
    }

    private void handleFailure(Exception error) {
        try {
            config.getOnFailure().handle((Exception) rootCause(error));
        } catch (Exception ignored) {
            // Best effort callback.
        }
    }

    private static Throwable rootCause(Throwable input) {
        Throwable current = input;
        while (current instanceof CompletionException && current.getCause() != null) {
            current = current.getCause();
        }
        return current;
    }

    private static List<Integer> utf8ToByteList(String input) {
        byte[] bytes = input.getBytes(StandardCharsets.UTF_8);
        List<Integer> output = new ArrayList<>(bytes.length);
        for (byte value : bytes) {
            output.add(value & 0xff);
        }
        return output;
    }
}
