import 'dart:async';
import 'dart:convert';
import 'dart:math';

import 'package:shelf/shelf.dart';
import 'package:shelf_router/shelf_router.dart';
import 'package:shelf_web_socket/shelf_web_socket.dart';
import 'package:web_socket_channel/io.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

import 'models.dart';
import 'signature_client.dart';

typedef OnSuccess =
    FutureOr<void> Function(
      VerifySignatureResponse verifyResponse,
      Map<String, Object?> session,
    );
typedef OnFailure =
    FutureOr<void> Function(Object error, StackTrace stackTrace);

class ShelfSignatureAdapter {
  ShelfSignatureAdapter({
    SignatureClient? signatureClient,
    this.requestedClaims = const <String>[],
    this.requireSitePass = false,
    String Function()? messageGenerator,
    OnSuccess? onSuccess,
    OnFailure? onFailure,
    this.expireTimeSeconds = 3600,
  }) : _signatureClient = signatureClient ?? SignatureClient.fromEnv(),
       _messageGenerator = messageGenerator ?? (() => ''),
       _onSuccess = onSuccess ?? _defaultOnSuccess,
       _onFailure = onFailure ?? _defaultOnFailure {
    _router.get('/signature/notbot', _handleSignatureNotbot);
    _router.get('/signature/status', _handleSignatureStatus);
    _router.post('/signature/notbot/<requestId>', _handleGeneratePresentation);
    _router.post('/signature/verify/<requestId>', _handleVerify);
    _router.get(
      '/signature/honestbot',
      _proxyWebsocket('/signature/honestbot', 'x-presentation-hash'),
    );
    _router.get(
      '/calculate_site_pass',
      _proxyWebsocket('/calculate_site_pass', 'x-site-pass'),
    );
  }

  static const _sessionCookieName = 'julia_session';
  static const _sessionAttrName = 'juliaSignatureVerification';

  final SignatureClient _signatureClient;
  final List<String> requestedClaims;
  final bool requireSitePass;
  final String Function() _messageGenerator;
  final OnSuccess _onSuccess;
  final OnFailure _onFailure;
  final int expireTimeSeconds;

  final Router _router = Router();
  final Map<String, String> _sessionSignatures = <String, String>{};
  final Map<String, Map<String, Object?>> _sessions =
      <String, Map<String, Object?>>{};
  final Random _random = Random.secure();

  Handler get handler => _router.call;

  Future<Response> _handleSignatureNotbot(Request request) async {
    final sessionContext = _resolveSession(request);
    sessionContext.session.remove(_sessionAttrName);

    try {
      final response = await _signatureClient.startSignature(
        StartSignatureRequest(
          requestedCredentials: requestedClaims,
          requireSitePass: requireSitePass,
          requiredAliasLauncher: null,
          requestedMessage: utf8.encode(_messageGenerator()),
          expires:
              DateTime.now().millisecondsSinceEpoch ~/ 1000 + expireTimeSeconds,
        ),
      );
      _sessionSignatures[response.requestId] = sessionContext.sessionId;
      return _jsonResponse(sessionContext, response.requestId);
    } catch (error, stackTrace) {
      await _onFailure(error, stackTrace);
      return _jsonError(sessionContext, 502, '$error');
    }
  }

  Future<Response> _handleSignatureStatus(Request request) async {
    final sessionContext = _resolveSession(request);
    final status = sessionContext.session[_sessionAttrName] != null;
    return _jsonResponse(sessionContext, status);
  }

  Future<Response> _handleGeneratePresentation(
    Request request,
    String requestId,
  ) async {
    final sessionContext = _resolveSession(request);

    final body = await request.readAsString();
    final json = body.isEmpty
        ? <String, dynamic>{}
        : jsonDecode(body) as Map<String, dynamic>;

    if (json['nonce'] is! String) {
      return _jsonError(sessionContext, 400, 'Missing payload.nonce');
    }

    try {
      final response = await _signatureClient.generatePresentation(
        GeneratePresentationRequest(
          requestId: requestId,
          nonce: json['nonce'] as String,
        ),
      );
      return _jsonResponse(sessionContext, response.toJson());
    } catch (error, stackTrace) {
      await _onFailure(error, stackTrace);
      return _jsonError(sessionContext, 502, '$error');
    }
  }

  Future<Response> _handleVerify(Request request, String requestId) async {
    final sessionContext = _resolveSession(request);

    final body = await request.readAsString();
    final json = body.isEmpty
        ? <String, dynamic>{}
        : jsonDecode(body) as Map<String, dynamic>;

    final presentation = json['presentation'];
    if (presentation is! List<dynamic>) {
      return _jsonError(sessionContext, 400, 'Missing payload.presentation');
    }

    final verifyRequest = VerifySignatureRequest(
      requestId: requestId,
      presentation: presentation.cast<int>(),
    );

    late VerifySignatureResponse verifyResponse;
    try {
      verifyResponse = await _signatureClient.verifyPresentation(verifyRequest);
    } catch (error, stackTrace) {
      await _onFailure(error, stackTrace);
      return _jsonError(sessionContext, 422, '$error');
    }

    final targetSessionId =
        _sessionSignatures.remove(requestId) ?? sessionContext.sessionId;
    final targetSession = _sessions.putIfAbsent(
      targetSessionId,
      () => <String, Object?>{},
    );

    try {
      await _onSuccess(verifyResponse, targetSession);
      return _responseWithSession(sessionContext, Response(204));
    } catch (error, stackTrace) {
      await _onFailure(error, stackTrace);
      return _jsonError(sessionContext, 500, '$error');
    }
  }

  Handler _proxyWebsocket(String upstreamPath, String headerName) {
    return (Request request) {
      final wsBase = _signatureClient.baseUrl
          .replaceFirst('https://', 'wss://')
          .replaceFirst('http://', 'ws://')
          .replaceAll(RegExp(r'/$'), '');
      final upstreamUri = Uri.parse('$wsBase$upstreamPath');
      final headerValue = request.headers[headerName];

      final wsHandler = webSocketHandler((WebSocketChannel clientSocket) {
        final upstream = IOWebSocketChannel.connect(
          upstreamUri,
          headers: headerValue == null
              ? null
              : <String, dynamic>{headerName: headerValue},
        );

        final upstreamSub = upstream.stream.listen(
          (message) {
            clientSocket.sink.add(message);
          },
          onError: (_) {
            clientSocket.sink.close();
          },
          onDone: () {
            clientSocket.sink.close();
          },
        );

        final clientSub = clientSocket.stream.listen(
          (message) {
            upstream.sink.add(message);
          },
          onError: (_) {
            upstream.sink.close();
          },
          onDone: () {
            upstream.sink.close();
          },
        );

        clientSocket.sink.done.whenComplete(() async {
          await upstreamSub.cancel();
          await clientSub.cancel();
        });
      });

      return wsHandler(request);
    };
  }

  SessionContext _resolveSession(Request request) {
    final sessionId = _extractSessionId(request) ?? _generateSessionId();
    final created = _extractSessionId(request) == null;
    final session = _sessions.putIfAbsent(sessionId, () => <String, Object?>{});
    return SessionContext(
      sessionId: sessionId,
      session: session,
      created: created,
    );
  }

  String? _extractSessionId(Request request) {
    final cookieHeader = request.headers['cookie'];
    if (cookieHeader == null || cookieHeader.isEmpty) {
      return null;
    }

    final parts = cookieHeader.split(';');
    for (final part in parts) {
      final keyValue = part.trim().split('=');
      if (keyValue.length == 2 && keyValue.first == _sessionCookieName) {
        return keyValue.last;
      }
    }
    return null;
  }

  String _generateSessionId() {
    final value = _random.nextInt(1 << 32).toRadixString(16);
    final timestamp = DateTime.now().millisecondsSinceEpoch.toRadixString(16);
    return '$timestamp$value';
  }

  Response _jsonResponse(SessionContext context, Object body) {
    return _responseWithSession(
      context,
      Response.ok(
        jsonEncode(body),
        headers: const {'content-type': 'application/json'},
      ),
    );
  }

  Response _jsonError(SessionContext context, int status, String message) {
    return _responseWithSession(
      context,
      Response(
        status,
        body: jsonEncode(<String, String>{'error': message}),
        headers: const {'content-type': 'application/json'},
      ),
    );
  }

  Response _responseWithSession(SessionContext context, Response response) {
    if (!context.created) {
      return response;
    }
    return response.change(
      headers: <String, String>{
        ...response.headers,
        'set-cookie':
            '$_sessionCookieName=${context.sessionId}; Path=/; HttpOnly',
      },
    );
  }

  static FutureOr<void> _defaultOnSuccess(
    VerifySignatureResponse verifyResponse,
    Map<String, Object?> session,
  ) {
    session[_sessionAttrName] = verifyResponse.toJson();
  }

  static FutureOr<void> _defaultOnFailure(
    Object _error,
    StackTrace _stackTrace,
  ) {}
}

class SessionContext {
  SessionContext({
    required this.sessionId,
    required this.session,
    required this.created,
  });

  final String sessionId;
  final Map<String, Object?> session;
  final bool created;
}
