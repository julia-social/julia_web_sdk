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
    public static final String GENDER = "julia://./v1/pii/gender";
    public static final String NATIONALITY = "julia://./v1/pii/nationality";
    public static final String BIRTH_DATE = "julia://./v1/pii/date_of_birth";
    public static final String BIRTH_DAY = "julia://./v1/pii/birth_day";
    public static final String BIRTH_MONTH = "julia://./v1/pii/birth_month";
    public static final String BIRTH_YEAR = "julia://./v1/pii/birth_year";
    public static final String AGE = "julia://./v1/pii/age";
    public static final String AGE_OVER_13 = "julia://./v1/pii/age_over_13";
    public static final String AGE_OVER_14 = "julia://./v1/pii/age_over_14";
    public static final String AGE_OVER_15 = "julia://./v1/pii/age_over_15";
    public static final String AGE_OVER_16 = "julia://./v1/pii/age_over_16";
    public static final String AGE_OVER_17 = "julia://./v1/pii/age_over_17";
    public static final String AGE_OVER_18 = "julia://./v1/pii/age_over_18";
    public static final String AGE_OVER_19 = "julia://./v1/pii/age_over_19";
    public static final String AGE_OVER_20 = "julia://./v1/pii/age_over_20";
    public static final String AGE_OVER_21 = "julia://./v1/pii/age_over_21";
    public static final String AGE_OVER_22 = "julia://./v1/pii/age_over_22";
    public static final String AGE_OVER_23 = "julia://./v1/pii/age_over_23";
    public static final String AGE_OVER_24 = "julia://./v1/pii/age_over_24";
    public static final String AGE_OVER_25 = "julia://./v1/pii/age_over_25";
    public static final String AGE_OVER_100 = "julia://./v1/pii/age_over_100";
    public static final String AGE_RANGE_20_TO_24 = "julia://./v1/pii/age_range_20_24";
    public static final String AGE_RANGE_25_TO_29 = "julia://./v1/pii/age_range_25_29";
    public static final String AGE_RANGE_30_TO_34 = "julia://./v1/pii/age_range_30_34";
    public static final String AGE_RANGE_35_TO_39 = "julia://./v1/pii/age_range_35_39";
    public static final String AGE_RANGE_40_TO_44 = "julia://./v1/pii/age_range_40_44";
    public static final String AGE_RANGE_45_TO_49 = "julia://./v1/pii/age_range_45_49";
    public static final String AGE_RANGE_50_TO_54 = "julia://./v1/pii/age_range_50_54";
    public static final String AGE_RANGE_55_TO_59 = "julia://./v1/pii/age_range_55_59";
    public static final String AGE_RANGE_60_TO_64 = "julia://./v1/pii/age_range_60_64";
    public static final String AGE_RANGE_65_TO_69 = "julia://./v1/pii/age_range_65_69";
    public static final String AGE_RANGE_70_TO_74 = "julia://./v1/pii/age_range_70_74";
    public static final String AGE_RANGE_75_TO_79 = "julia://./v1/pii/age_range_75_79";
    public static final String AGE_RANGE_80_TO_84 = "julia://./v1/pii/age_range_80_84";
    public static final String AGE_RANGE_85_TO_89 = "julia://./v1/pii/age_range_85_89";
    public static final String AGE_RANGE_90_TO_94 = "julia://./v1/pii/age_range_90_94";
    public static final String AGE_RANGE_95_TO_99 = "julia://./v1/pii/age_range_95_99";
    public static final String AGE_RANGE_20_TO_29 = "julia://./v1/pii/age_range_20_29";
    public static final String AGE_RANGE_30_TO_39 = "julia://./v1/pii/age_range_30_39";
    public static final String AGE_RANGE_40_TO_49 = "julia://./v1/pii/age_range_40_49";
    public static final String AGE_RANGE_50_TO_59 = "julia://./v1/pii/age_range_50_59";
    public static final String AGE_RANGE_60_TO_69 = "julia://./v1/pii/age_range_60_69";
    public static final String AGE_RANGE_70_TO_79 = "julia://./v1/pii/age_range_70_79";
    public static final String AGE_RANGE_80_TO_89 = "julia://./v1/pii/age_range_80_89";
    public static final String AGE_RANGE_90_TO_99 = "julia://./v1/pii/age_range_90_99";

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
            Map.entry("Gender", GENDER),
            Map.entry("Nationality", NATIONALITY),
            Map.entry("BirthDate", BIRTH_DATE),
            Map.entry("BirthDay", BIRTH_DAY),
            Map.entry("BirthMonth", BIRTH_MONTH),
            Map.entry("BirthYear", BIRTH_YEAR),
            Map.entry("Age", AGE),
            Map.entry("AgeOver13", AGE_OVER_13),
            Map.entry("AgeOver14", AGE_OVER_14),
            Map.entry("AgeOver15", AGE_OVER_15),
            Map.entry("AgeOver16", AGE_OVER_16),
            Map.entry("AgeOver17", AGE_OVER_17),
            Map.entry("AgeOver18", AGE_OVER_18),
            Map.entry("AgeOver19", AGE_OVER_19),
            Map.entry("AgeOver20", AGE_OVER_20),
            Map.entry("AgeOver21", AGE_OVER_21),
            Map.entry("AgeOver22", AGE_OVER_22),
            Map.entry("AgeOver23", AGE_OVER_23),
            Map.entry("AgeOver24", AGE_OVER_24),
            Map.entry("AgeOver25", AGE_OVER_25),
            Map.entry("AgeOver100", AGE_OVER_100),
            Map.entry("AgeRange20To24", AGE_RANGE_20_TO_24),
            Map.entry("AgeRange25To29", AGE_RANGE_25_TO_29),
            Map.entry("AgeRange30To34", AGE_RANGE_30_TO_34),
            Map.entry("AgeRange35To39", AGE_RANGE_35_TO_39),
            Map.entry("AgeRange40To44", AGE_RANGE_40_TO_44),
            Map.entry("AgeRange45To49", AGE_RANGE_45_TO_49),
            Map.entry("AgeRange50To54", AGE_RANGE_50_TO_54),
            Map.entry("AgeRange55To59", AGE_RANGE_55_TO_59),
            Map.entry("AgeRange60To64", AGE_RANGE_60_TO_64),
            Map.entry("AgeRange65To69", AGE_RANGE_65_TO_69),
            Map.entry("AgeRange70To74", AGE_RANGE_70_TO_74),
            Map.entry("AgeRange75To79", AGE_RANGE_75_TO_79),
            Map.entry("AgeRange80To84", AGE_RANGE_80_TO_84),
            Map.entry("AgeRange85To89", AGE_RANGE_85_TO_89),
            Map.entry("AgeRange90To94", AGE_RANGE_90_TO_94),
            Map.entry("AgeRange95To99", AGE_RANGE_95_TO_99),
            Map.entry("AgeRange20To29", AGE_RANGE_20_TO_29),
            Map.entry("AgeRange30To39", AGE_RANGE_30_TO_39),
            Map.entry("AgeRange40To49", AGE_RANGE_40_TO_49),
            Map.entry("AgeRange50To59", AGE_RANGE_50_TO_59),
            Map.entry("AgeRange60To69", AGE_RANGE_60_TO_69),
            Map.entry("AgeRange70To79", AGE_RANGE_70_TO_79),
            Map.entry("AgeRange80To89", AGE_RANGE_80_TO_89),
            Map.entry("AgeRange90To99", AGE_RANGE_90_TO_99)
    );
}
