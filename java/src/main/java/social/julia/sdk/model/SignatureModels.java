package social.julia.sdk.model;

import com.fasterxml.jackson.annotation.JsonProperty;

import java.util.ArrayList;
import java.util.List;

public final class SignatureModels {
    private SignatureModels() {
    }

    public static class StartSignatureRequest {
        @JsonProperty("requested_credentials")
        public List<String> requestedCredentials = new ArrayList<>();

        @JsonProperty("require_site_pass")
        public boolean requireSitePass;

        @JsonProperty("required_alias_launcher")
        public String requiredAliasLauncher;

        @JsonProperty("requested_message")
        public List<Integer> requestedMessage = new ArrayList<>();

        @JsonProperty("expires")
        public long expires;
    }

    public static class StartSignatureResponse {
        @JsonProperty("request_id")
        public String requestId;
    }

    public static class GeneratePresentationRequest {
        @JsonProperty("request_id")
        public String requestId;

        @JsonProperty("nonce")
        public String nonce;
    }

    public static class GeneratePresentationResponse {
        @JsonProperty("compressed_presentation")
        public List<Integer> compressedPresentation = new ArrayList<>();
    }

    public static class ServerPresentation {
        @JsonProperty("compressed_presentation")
        public List<Integer> compressedPresentation = new ArrayList<>();
    }

    public static class ClientPresentation {
        @JsonProperty("presentation")
        public List<Integer> presentation = new ArrayList<>();
    }

    public static class VerifySignatureRequest {
        @JsonProperty("request_id")
        public String requestId;

        @JsonProperty("presentation")
        public List<Integer> presentation = new ArrayList<>();
    }

    public static class SignatureRequest {
        @JsonProperty("nonce")
        public String nonce;
    }

    public static class Claim {
        @JsonProperty("property")
        public String property;

        @JsonProperty("value")
        public List<Integer> value = new ArrayList<>();
    }

    public static class DIDInfo {
        @JsonProperty("launcher_id")
        public List<Integer> launcherId = new ArrayList<>();
    }

    public static class VerifySignatureResponse {
        @JsonProperty("alias_did")
        public DIDInfo aliasDid = new DIDInfo();

        @JsonProperty("site_pass")
        public String sitePass;

        @JsonProperty("claims")
        public List<Claim> claims = new ArrayList<>();

        @JsonProperty("timestamp")
        public long timestamp;

        @JsonProperty("presentation")
        public List<Integer> presentation = new ArrayList<>();
    }
}
