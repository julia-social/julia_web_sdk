import 'package:julia_web_sdk/julia_web_sdk.dart';

Future<void> main() async {
  final client = SignatureClient(baseUrl: 'https://example.com');

  final requestId = await client.getSignatureRequestId();
  print('request id: $requestId');

  final status = await client.getSignatureStatus();
  print('signature status: $status');

  // Placeholder values for local testing.
  // In production, not.bot provides nonce + presentation values.
  const nonce =
      '0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff';
  final presentation = await client.generateSignaturePresentation(requestId, nonce);
  print(
    'presentation bytes: ${presentation.compressedPresentation.length}',
  );
  await client.verifySignaturePresentation(requestId, <int>[]);
}
