package social.julia.sdk.server;

import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpSession;
import social.julia.sdk.model.SignatureModels.VerifySignatureResponse;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.function.Supplier;

public class AuthAdapterConfig {
    @FunctionalInterface
    public interface OnSuccess {
        void handle(VerifySignatureResponse response, HttpSession session) throws Exception;
    }

    @FunctionalInterface
    public interface OnFailure {
        void handle(Exception error) throws Exception;
    }

    @FunctionalInterface
    public interface SessionResolver {
        HttpSession resolve(String sessionId, HttpServletRequest request);
    }

    private final List<String> requestedClaims;
    private final boolean requireSitePass;
    private final Supplier<String> messageGenerator;
    private final OnSuccess onSuccess;
    private final OnFailure onFailure;
    private final long expireTimeSeconds;
    private final String sessionAttributeName;
    private final SessionResolver sessionResolver;

    private AuthAdapterConfig(Builder builder) {
        this.requestedClaims = builder.requestedClaims;
        this.requireSitePass = builder.requireSitePass;
        this.messageGenerator = builder.messageGenerator;
        this.onSuccess = builder.onSuccess;
        this.onFailure = builder.onFailure;
        this.expireTimeSeconds = builder.expireTimeSeconds;
        this.sessionAttributeName = builder.sessionAttributeName;
        this.sessionResolver = builder.sessionResolver;
    }

    public static Builder builder() {
        return new Builder();
    }

    public List<String> getRequestedClaims() {
        return requestedClaims;
    }

    public boolean isRequireSitePass() {
        return requireSitePass;
    }

    public Supplier<String> getMessageGenerator() {
        return messageGenerator;
    }

    public OnSuccess getOnSuccess() {
        return onSuccess;
    }

    public OnFailure getOnFailure() {
        return onFailure;
    }

    public long getExpireTimeSeconds() {
        return expireTimeSeconds;
    }

    public String getSessionAttributeName() {
        return sessionAttributeName;
    }

    public SessionResolver getSessionResolver() {
        return sessionResolver;
    }

    public static final class Builder {
        private final List<String> requestedClaims = new ArrayList<>();
        private boolean requireSitePass;
        private Supplier<String> messageGenerator = () -> "";
        private OnSuccess onSuccess = (response, session) -> session.setAttribute("juliaSignatureVerification", response);
        private OnFailure onFailure = _error -> {
        };
        private long expireTimeSeconds = 3600;
        private String sessionAttributeName = "juliaSignatureVerification";
        private SessionResolver sessionResolver = (_sessionId, request) -> request.getSession(false);

        public Builder requestedClaims(List<String> values) {
            this.requestedClaims.clear();
            if (values != null) {
                this.requestedClaims.addAll(values);
            }
            return this;
        }

        public Builder requireSitePass(boolean value) {
            this.requireSitePass = value;
            return this;
        }

        public Builder messageGenerator(Supplier<String> value) {
            this.messageGenerator = Objects.requireNonNull(value);
            return this;
        }

        public Builder onSuccess(OnSuccess value) {
            this.onSuccess = Objects.requireNonNull(value);
            return this;
        }

        public Builder onFailure(OnFailure value) {
            this.onFailure = Objects.requireNonNull(value);
            return this;
        }

        public Builder expireTimeSeconds(long value) {
            this.expireTimeSeconds = value;
            return this;
        }

        public Builder sessionAttributeName(String value) {
            this.sessionAttributeName = Objects.requireNonNull(value);
            return this;
        }

        public Builder sessionResolver(SessionResolver value) {
            this.sessionResolver = Objects.requireNonNull(value);
            return this;
        }

        public AuthAdapterConfig build() {
            return new AuthAdapterConfig(this);
        }
    }
}
