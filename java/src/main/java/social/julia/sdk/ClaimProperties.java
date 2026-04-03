package social.julia.sdk;

import java.util.Map;

public final class ClaimProperties {
    private ClaimProperties() {
    }

    public static final String ACCOUNT_NAME = "sha2-256|CBOR://./v1/account_name";
    public static final String DOMAIN_NAME = "sha2-256|CBOR://./v1/domain_name";
    public static final String SIGNATURE_LICENSE = "notbot://./v1/signature_license";
    public static final String SITE_PASS = "julia://./v1/site_pass";
    public static final String HONESTBOT_0 = "notbot://./v1/honestbot0";
    public static final String NOTBOT_0 = "notbot://./v1/notbot0";
    public static final String NOTBOT_1 = "notbot://./v1/notbot1";
    public static final String NOTBOT_2 = "notbot://./v1/notbot2";
    public static final String FIRST_NAME = "julia://./v1/pii/first_name";
    public static final String GIVEN_NAMES = "julia://./v1/pii/given_names";
    public static final String FAMILY_NAME = "julia://./v1/pii/family_name";
    public static final String AGE_OVER_18 = "julia://./v1/pii/age_over_18";

    public static final Map<String, String> ALL = Map.ofEntries(
            Map.entry("AccountName", ACCOUNT_NAME),
            Map.entry("DomainName", DOMAIN_NAME),
            Map.entry("SignatureLicense", SIGNATURE_LICENSE),
            Map.entry("SitePass", SITE_PASS),
            Map.entry("Honestbot0", HONESTBOT_0),
            Map.entry("Notbot0", NOTBOT_0),
            Map.entry("Notbot1", NOTBOT_1),
            Map.entry("Notbot2", NOTBOT_2),
            Map.entry("FirstName", FIRST_NAME),
            Map.entry("GivenNames", GIVEN_NAMES),
            Map.entry("FamilyName", FAMILY_NAME),
            Map.entry("AgeOver18", AGE_OVER_18)
    );
}
