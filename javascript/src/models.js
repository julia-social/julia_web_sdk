export const ClaimProperties = Object.freeze({
  AccountName: "sha2-256|CBOR://./v1/account_name",
  DomainName: "sha2-256|CBOR://./v1/domain_name",
  SitePass: "julia://./v1/site_pass",
  Honestbot0: "notbot://./v1/honestbot0",
  Notbot0: "notbot://./v1/notbot0",
  Notbot1: "notbot://./v1/notbot1",
  Notbot2: "notbot://./v1/notbot2",
  SignatureLicense: "notbot://./v1/signature_license",
  GivenNames: "julia://./v1/pii/given_names",
  FamilyName: "julia://./v1/pii/family_name",
  FirstName: "julia://./v1/pii/first_name",
  Gender: "julia://./v1/pii/gender",
  Nationality: "julia://./v1/pii/nationality",
  BirthDate: "julia://./v1/pii/date_of_birth",
  BirthDay: "julia://./v1/pii/birth_day",
  BirthMonth: "julia://./v1/pii/birth_month",
  BirthYear: "julia://./v1/pii/birth_year",
  Age: "julia://./v1/pii/age",
  AgeOver18: "julia://./v1/pii/age_over_18"
});

export function bytes32(value) {
  if (typeof value !== "string" || !/^0x[0-9a-fA-F]{64}$/.test(value)) {
    throw new TypeError("Expected a Bytes32 hex string (0x + 64 hex chars)");
  }
  return value.toLowerCase();
}

export function byteArray(value) {
  if (!Array.isArray(value) || !value.every((v) => Number.isInteger(v) && v >= 0 && v <= 255)) {
    throw new TypeError("Expected an array of byte integers (0-255)");
  }
  return value;
}
