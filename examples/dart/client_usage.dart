import 'package:julia_web_sdk/julia_web_sdk.dart';

Future<void> main() async {
  final client = SignatureClient(baseUrl: 'https://example.com');

  final requestId = await client.getAuthRequestId();
  print('request id: $requestId');

  final status = await client.getAuthStatus();
  print('authenticated: $status');

  // Placeholder values for local testing.
  // In production, not.bot provides nonce + presentation values.
  const nonce =
      '0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff';
  final presentation = await client.generateAuthPresentation(requestId, nonce);
  print(
    'presentation bytes: ${presentation.compressedPresentation.length}',
  );
  await client.verifyAuthPresentation(requestId, <int>[]);
}
