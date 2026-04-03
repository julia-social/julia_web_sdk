import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:http/http.dart' as http;

import 'models.dart';

class JuliaWebSdkException implements Exception {
  JuliaWebSdkException(this.message, {this.statusCode, this.body});

  final String message;
  final int? statusCode;
  final Object? body;

  @override
  String toString() => 'JuliaWebSdkException($statusCode): $message';
}

class SignatureClient {
  SignatureClient({
    required this.baseUrl,
    this.apiKey,
    http.Client? httpClient,
    this.timeout = const Duration(seconds: 180),
  }) : _httpClient = httpClient ?? http.Client();

  factory SignatureClient.fromEnv({http.Client? httpClient}) {
    final host = Platform.environment['SIGNATURE_HOSTNAME'] ?? 'localhost';
    final port =
        int.tryParse(Platform.environment['SIGNATURE_PORT'] ?? '8080') ?? 8080;
    final apiKey = Platform.environment['SIGNATURE_API_KEY'] ?? 'CHANGE_ME';
    final scheme = host == 'localhost' ? 'http' : 'https';
    return SignatureClient(
      baseUrl: '$scheme://$host:$port',
      apiKey: apiKey,
      httpClient: httpClient,
    );
  }

  final String baseUrl;
  final String? apiKey;
  final Duration timeout;
  final http.Client _httpClient;

  Future<StartSignatureResponse> startSignature(
    StartSignatureRequest request,
  ) async {
    final json = await _requestMap(
      'POST',
      '/signature/start',
      body: request.toJson(),
      includeApiKey: true,
      serviceLabel: 'Signature',
    );
    return StartSignatureResponse.fromJson(json);
  }

  Future<GeneratePresentationResponse> generatePresentation(
    GeneratePresentationRequest request,
  ) async {
    final json = await _requestMap(
      'POST',
      '/signature/presentation',
      body: request.toJson(),
      includeApiKey: true,
      serviceLabel: 'Signature',
    );
    return GeneratePresentationResponse.fromJson(json);
  }

  Future<VerifySignatureResponse> verifyPresentation(
    VerifySignatureRequest request,
  ) async {
    final json = await _requestMap(
      'POST',
      '/signature/verify',
      body: request.toJson(),
      includeApiKey: true,
      serviceLabel: 'Signature',
    );
    return VerifySignatureResponse.fromJson(json);
  }

  Future<String> getAuthRequestId() async {
    final data = await _request('GET', '/auth/notbot', serviceLabel: 'Auth');
    if (data is String) {
      return data;
    }
    throw JuliaWebSdkException(
      'Invalid /auth/notbot response type',
      body: data,
    );
  }

  Future<bool> getAuthStatus() async {
    final data = await _request('GET', '/auth/status', serviceLabel: 'Auth');
    if (data is bool) {
      return data;
    }
    throw JuliaWebSdkException('Invalid /auth/status response type', body: data);
  }

  Future<GeneratePresentationResponse> generateAuthPresentation(
    String requestId,
    String nonce,
  ) async {
    final response = await _requestMap(
      'POST',
      '/auth/notbot/${Uri.encodeComponent(requestId)}',
      body: SignatureRequest(nonce: nonce).toJson(),
      serviceLabel: 'Auth',
    );
    return GeneratePresentationResponse.fromJson(response);
  }

  Future<void> verifyAuthPresentation(
    String requestId,
    List<int> presentation,
  ) async {
    await _request(
      'POST',
      '/auth/verify/${Uri.encodeComponent(requestId)}',
      body: ClientPresentation(presentation: presentation).toJson(),
      serviceLabel: 'Auth',
    );
  }

  Future<Map<String, dynamic>> _requestMap(
    String method,
    String path, {
    Map<String, dynamic>? body,
    bool includeApiKey = false,
    required String serviceLabel,
  }) async {
    final data = await _request(
      method,
      path,
      body: body,
      includeApiKey: includeApiKey,
      serviceLabel: serviceLabel,
    );
    if (data is Map<String, dynamic>) {
      return data;
    }
    throw JuliaWebSdkException(
      '$serviceLabel service returned non-object response',
      body: data,
    );
  }

  Future<dynamic> _request(
    String method,
    String path, {
    Map<String, dynamic>? body,
    bool includeApiKey = false,
    required String serviceLabel,
  }) async {
    final url = Uri.parse('${_normalizeBase(baseUrl)}$path');
    final headers = <String, String>{'content-type': 'application/json'};
    if (includeApiKey) {
      final key = apiKey;
      if (key == null || key.isEmpty) {
        throw JuliaWebSdkException('apiKey is required for signature endpoints');
      }
      headers['api-key'] = key;
    }

    late http.Response response;
    try {
      final request = http.Request(method, url)..headers.addAll(headers);
      if (body != null) {
        request.body = jsonEncode(body);
      }
      final streamed = await _httpClient.send(request).timeout(timeout);
      response = await http.Response.fromStream(streamed);
    } catch (error) {
      throw JuliaWebSdkException(
        'Error connecting to ${serviceLabel.toLowerCase()} service: $error',
      );
    }

    final raw = response.body;
    dynamic decoded;
    try {
      decoded = raw.isEmpty ? null : jsonDecode(raw);
    } catch (error) {
      throw JuliaWebSdkException(
        'Error parsing ${serviceLabel.toLowerCase()} response JSON: $error - $raw',
        statusCode: response.statusCode,
        body: raw,
      );
    }

    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw JuliaWebSdkException(
        '$serviceLabel service error ${response.statusCode}: $raw',
        statusCode: response.statusCode,
        body: decoded,
      );
    }

    return decoded;
  }
}

String _normalizeBase(String input) =>
    input.endsWith('/') ? input.substring(0, input.length - 1) : input;
