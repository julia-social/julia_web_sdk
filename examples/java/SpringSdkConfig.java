package examples.java;

import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import social.julia.sdk.ClaimProperties;
import social.julia.sdk.client.SignatureClient;
import social.julia.sdk.server.AuthAdapterConfig;
import social.julia.sdk.server.SpringAuthController;

import java.util.List;

@Configuration
public class SpringSdkConfig {
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
                .onFailure(error -> System.err.println("verification error " + error.getMessage()))
                .build();
    }

    @Bean
    public SpringAuthController springAuthController(SignatureClient client, AuthAdapterConfig config) {
        return new SpringAuthController(client, config);
    }
}
