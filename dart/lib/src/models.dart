class StartSignatureRequest {
  StartSignatureRequest({
    required this.requestedCredentials,
    required this.requireSitePass,
    required this.requiredAliasLauncher,
    required this.requestedMessage,
    required this.expires,
  });

  final List<String> requestedCredentials;
  final bool requireSitePass;
  final String? requiredAliasLauncher;
  final List<int> requestedMessage;
  final int expires;

  Map<String, dynamic> toJson() => {
    'requested_credentials': requestedCredentials,
    'require_site_pass': requireSitePass,
    'required_alias_launcher': requiredAliasLauncher,
    'requested_message': requestedMessage,
    'expires': expires,
  };
}

class StartSignatureResponse {
  StartSignatureResponse({required this.requestId});

  final String requestId;

  factory StartSignatureResponse.fromJson(Map<String, dynamic> json) =>
      StartSignatureResponse(requestId: json['request_id'] as String);
}

class GeneratePresentationRequest {
  GeneratePresentationRequest({required this.requestId, required this.nonce});

  final String requestId;
  final String nonce;

  Map<String, dynamic> toJson() => {'request_id': requestId, 'nonce': nonce};
}

class GeneratePresentationResponse {
  GeneratePresentationResponse({required this.compressedPresentation});

  final List<int> compressedPresentation;

  factory GeneratePresentationResponse.fromJson(Map<String, dynamic> json) =>
      GeneratePresentationResponse(
        compressedPresentation:
            (json['compressed_presentation'] as List<dynamic>).cast<int>(),
      );

  Map<String, dynamic> toJson() => {
    'compressed_presentation': compressedPresentation,
  };
}

class SignatureRequest {
  SignatureRequest({required this.nonce});

  final String nonce;

  Map<String, dynamic> toJson() => {'nonce': nonce};
}

class ClientPresentation {
  ClientPresentation({required this.presentation});

  final List<int> presentation;

  Map<String, dynamic> toJson() => {'presentation': presentation};
}

class Claim {
  Claim({required this.property, required this.value});

  final String property;
  final List<int> value;

  factory Claim.fromJson(Map<String, dynamic> json) => Claim(
    property: json['property'] as String,
    value: (json['value'] as List<dynamic>).cast<int>(),
  );

  Map<String, dynamic> toJson() => {'property': property, 'value': value};
}

class DidInfo {
  DidInfo({required this.launcherId});

  final List<int> launcherId;

  factory DidInfo.fromJson(Map<String, dynamic> json) =>
      DidInfo(launcherId: (json['launcher_id'] as List<dynamic>).cast<int>());

  Map<String, dynamic> toJson() => {'launcher_id': launcherId};
}

class VerifySignatureRequest {
  VerifySignatureRequest({required this.requestId, required this.presentation});

  final String requestId;
  final List<int> presentation;

  Map<String, dynamic> toJson() => {
    'request_id': requestId,
    'presentation': presentation,
  };
}

class VerifySignatureResponse {
  VerifySignatureResponse({
    required this.aliasDid,
    required this.sitePass,
    required this.claims,
    required this.timestamp,
    required this.presentation,
  });

  final DidInfo aliasDid;
  final String? sitePass;
  final List<Claim> claims;
  final int timestamp;
  final List<int> presentation;

  factory VerifySignatureResponse.fromJson(Map<String, dynamic> json) =>
      VerifySignatureResponse(
        aliasDid: DidInfo.fromJson(json['alias_did'] as Map<String, dynamic>),
        sitePass: json['site_pass'] as String?,
        claims: (json['claims'] as List<dynamic>)
            .map((entry) => Claim.fromJson(entry as Map<String, dynamic>))
            .toList(),
        timestamp: json['timestamp'] as int,
        presentation: (json['presentation'] as List<dynamic>).cast<int>(),
      );

  Map<String, dynamic> toJson() => {
    'alias_did': aliasDid.toJson(),
    'site_pass': sitePass,
    'claims': claims.map((claim) => claim.toJson()).toList(),
    'timestamp': timestamp,
    'presentation': presentation,
  };
}
