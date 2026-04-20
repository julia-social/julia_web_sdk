package social.julia.sdk.server;

import org.springframework.context.annotation.Configuration;
import org.springframework.web.socket.config.annotation.EnableWebSocket;
import org.springframework.web.socket.config.annotation.WebSocketConfigurer;
import org.springframework.web.socket.config.annotation.WebSocketHandlerRegistry;
import social.julia.sdk.client.SignatureClient;

import java.net.URI;

@Configuration
@EnableWebSocket
public class JuliaWebSocketConfig implements WebSocketConfigurer {
    private final SignatureClient signatureClient;

    public JuliaWebSocketConfig(SignatureClient signatureClient) {
        this.signatureClient = signatureClient;
    }

    @Override
    public void registerWebSocketHandlers(WebSocketHandlerRegistry registry) {
        String base = signatureClient.getBaseUrl()
                .replace("https://", "wss://")
                .replace("http://", "ws://");

        registry.addHandler(
                        new JuliaWebSocketProxyHandler(
                                URI.create(base + "/signature/honestbot"),
                                "x-presentation-hash"
                        ),
                        "/signature/honestbot"
                )
                .setAllowedOriginPatterns("*");

        registry.addHandler(
                        new JuliaWebSocketProxyHandler(
                                URI.create(base + "/calculate_site_pass"),
                                "x-site-pass"
                        ),
                        "/calculate_site_pass"
                )
                .setAllowedOriginPatterns("*");
    }
}
