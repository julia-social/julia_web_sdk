package social.julia.sdk.server;

import org.springframework.web.socket.BinaryMessage;
import org.springframework.web.socket.CloseStatus;
import org.springframework.web.socket.PongMessage;
import org.springframework.web.socket.TextMessage;
import org.springframework.web.socket.WebSocketSession;
import org.springframework.web.socket.handler.AbstractWebSocketHandler;

import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.WebSocket;
import java.nio.ByteBuffer;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.ConcurrentHashMap;

public class JuliaWebSocketProxyHandler extends AbstractWebSocketHandler {
    private final HttpClient webSocketClient;
    private final URI upstreamUri;
    private final String forwardedHeaderName;
    private final Map<String, WebSocket> upstreamSockets = new ConcurrentHashMap<>();

    public JuliaWebSocketProxyHandler(URI upstreamUri, String forwardedHeaderName) {
        this.webSocketClient = HttpClient.newHttpClient();
        this.upstreamUri = upstreamUri;
        this.forwardedHeaderName = forwardedHeaderName;
    }

    @Override
    public void afterConnectionEstablished(WebSocketSession session) {
        var builder = webSocketClient.newWebSocketBuilder();
        String forwardedHeader = session.getHandshakeHeaders().getFirst(forwardedHeaderName);
        if (forwardedHeader != null && !forwardedHeader.isBlank()) {
            builder.header(forwardedHeaderName, forwardedHeader);
        }

        builder.buildAsync(upstreamUri, new UpstreamListener(session))
                .thenAccept(webSocket -> upstreamSockets.put(session.getId(), webSocket))
                .join();
    }

    @Override
    protected void handleTextMessage(WebSocketSession session, TextMessage message) {
        WebSocket upstream = upstreamSockets.get(session.getId());
        if (upstream != null) {
            upstream.sendText(message.getPayload(), true);
        }
    }

    @Override
    protected void handleBinaryMessage(WebSocketSession session, BinaryMessage message) {
        WebSocket upstream = upstreamSockets.get(session.getId());
        if (upstream != null) {
            upstream.sendBinary(message.getPayload(), true);
        }
    }

    @Override
    protected void handlePongMessage(WebSocketSession session, PongMessage message) {
        WebSocket upstream = upstreamSockets.get(session.getId());
        if (upstream != null) {
            upstream.sendPong(message.getPayload());
        }
    }

    @Override
    public void afterConnectionClosed(WebSocketSession session, CloseStatus status) {
        WebSocket upstream = upstreamSockets.remove(session.getId());
        if (upstream != null) {
            upstream.sendClose(status.getCode(), status.getReason());
        }
    }

    @Override
    public void handleTransportError(WebSocketSession session, Throwable exception) throws Exception {
        afterConnectionClosed(session, CloseStatus.SERVER_ERROR);
    }

    private final class UpstreamListener implements WebSocket.Listener {
        private final WebSocketSession downstream;

        private UpstreamListener(WebSocketSession downstream) {
            this.downstream = downstream;
        }

        @Override
        public void onOpen(WebSocket webSocket) {
            webSocket.request(1);
        }

        @Override
        public CompletionStage<?> onText(WebSocket webSocket, CharSequence data, boolean last) {
            if (last) {
                synchronized (downstream) {
                    try {
                        if (downstream.isOpen()) {
                            downstream.sendMessage(new TextMessage(data.toString()));
                        }
                    } catch (IOException ignored) {
                        // Close handled by transport callbacks.
                    }
                }
            }
            webSocket.request(1);
            return CompletableFuture.completedFuture(null);
        }

        @Override
        public CompletionStage<?> onBinary(WebSocket webSocket, ByteBuffer data, boolean last) {
            if (last) {
                byte[] bytes = new byte[data.remaining()];
                data.get(bytes);
                synchronized (downstream) {
                    try {
                        if (downstream.isOpen()) {
                            downstream.sendMessage(new BinaryMessage(bytes));
                        }
                    } catch (IOException ignored) {
                        // Close handled by transport callbacks.
                    }
                }
            }
            webSocket.request(1);
            return CompletableFuture.completedFuture(null);
        }

        @Override
        public CompletionStage<?> onClose(WebSocket webSocket, int statusCode, String reason) {
            synchronized (downstream) {
                try {
                    if (downstream.isOpen()) {
                        downstream.close(new CloseStatus(statusCode, reason));
                    }
                } catch (IOException ignored) {
                    // Connection already dropped.
                }
            }
            return CompletableFuture.completedFuture(null);
        }

        @Override
        public void onError(WebSocket webSocket, Throwable error) {
            synchronized (downstream) {
                try {
                    if (downstream.isOpen()) {
                        downstream.close(CloseStatus.SERVER_ERROR);
                    }
                } catch (IOException ignored) {
                    // Connection already dropped.
                }
            }
        }
    }
}
