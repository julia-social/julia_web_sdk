import 'dart:io';

import 'package:julia_web_sdk/julia_web_sdk.dart';
import 'package:shelf/shelf_io.dart' as shelf_io;

Future<void> main() async {
  final adapter = ShelfSignatureAdapter(
    signatureClient: SignatureClient.fromEnv(),
    requestedClaims: [
      claimProperties['Notbot0']!,
      claimProperties['SitePass']!,
      claimProperties['FirstName']!,
      claimProperties['AgeOver18']!,
    ],
    requireSitePass: true,
    messageGenerator: () => 'Verifying My Identity with example.com',
    onFailure: (error, stackTrace) {
      stderr.writeln('verification error: $error');
      stderr.writeln(stackTrace);
    },
  );

  final server = await shelf_io.serve(
    adapter.handler,
    InternetAddress.anyIPv4,
    3000,
  );
  stdout.writeln('Server running on port ${server.port}');
}
