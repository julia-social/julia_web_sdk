# Julia Web SDK (Dart)

Dart package for Julia web verification flows.

## Install

```bash
cd dart
dart pub get
```

## What this package provides

- `SignatureClient`: async client for both `/signature/*` and `/auth/*`.
- `ShelfAuthAdapter`: shelf adapter with auth routes + websocket proxy handlers.
- `claimProperties`: claim constants.

## Recommended Integration

- Use `ShelfAuthAdapter` in your backend service as the default integration path.
- Use `SignatureClient` directly for custom/manual endpoint calls or automation scripts.

## Client Methods

- Auth:
  - `getAuthRequestId()`
  - `getAuthStatus()`
  - `generateAuthPresentation(requestId, nonce)`
  - `verifyAuthPresentation(requestId, presentation)`
- Signature:
  - `startSignature(request)`
  - `generatePresentation(request)`
  - `verifyPresentation(request)`

## Shelf Server Integration

```dart
import 'dart:io';

import 'package:julia_web_sdk/julia_web_sdk.dart';
import 'package:shelf/shelf_io.dart' as shelf_io;

Future<void> main() async {
  final adapter = ShelfAuthAdapter(
    signatureClient: SignatureClient.fromEnv(),
    requestedClaims: [
      claimProperties['Notbot0']!,
      claimProperties['SitePass']!,
      claimProperties['FirstName']!,
      claimProperties['AgeOver18']!,
    ],
    requireSitePass: true,
    messageGenerator: () => 'Verifying My Identity with example.com',
  );

  final server = await shelf_io.serve(adapter.handler, InternetAddress.anyIPv4, 3000);
  print('Listening on port ${server.port}');
}
```

## Client Usage

```dart
final client = SignatureClient(baseUrl: 'https://example.com');
final requestId = await client.getAuthRequestId();
final status = await client.getAuthStatus();
final presentation = await client.generateAuthPresentation(
  requestId,
  '0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff',
);
await client.verifyAuthPresentation(requestId, <int>[]);
```

## Examples

- `../examples/dart/shelf_server.dart`
- `../examples/dart/client_usage.dart`
