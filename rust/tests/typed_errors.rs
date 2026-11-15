use julia_web_sdk::error::SignatureFailure;
use julia_web_sdk::models::{GeneratePresentationRequest, VerifySignatureRequest};
use julia_web_sdk::signature_client::SignatureClient;
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread::{self, JoinHandle};
use std::time::Duration;

const MEDIA: &str = "application/vnd.notbot.error+json";
const CORRELATION: &str = "b8621e52-7524-4cac-9991-893fbe6bf7a2";
const SECRET: &str = "SECRET private credential /database/password";

// Exercise the public client against real HTTP, without adding a mock dependency.
fn mock(status: u16, headers: &str, body: &str) -> (SignatureClient, JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let response = format!(
        "HTTP/1.1 {status} Mock\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let task = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = Vec::new();
        loop {
            let mut buf = [0; 4096];
            let count = stream.read(&mut buf).unwrap();
            assert_ne!(count, 0, "request closed before completion");
            request.extend_from_slice(&buf[..count]);
            if let Some(end) = request.windows(4).position(|part| part == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&request[..end]);
                let length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                if request.len() >= end + 4 + length {
                    break;
                }
            }
        }
        // The client may reject invalid headers without consuming the body.
        let _ = stream.write_all(response.as_bytes());
        String::from_utf8(request).unwrap()
    });
    (
        SignatureClient::with_url_and_api_key(&url, "test-api-key"),
        task,
    )
}

fn envelope(code: &str) -> Value {
    json!({"version": 1, "code": code, "message": SECRET, "correlation_id": CORRELATION})
}

fn generate_request() -> GeneratePresentationRequest {
    GeneratePresentationRequest {
        request_id: "request-1".into(),
        nonce: Default::default(),
    }
}

fn verify_request() -> VerifySignatureRequest {
    VerifySignatureRequest {
        request_id: "request-2".into(),
        presentation: vec![1, 2, 3],
    }
}

fn assert_internal(failure: &SignatureFailure, status: Option<u16>) {
    assert_eq!(failure.version, 1);
    assert_eq!(failure.code, "ServerError");
    assert_eq!(failure.status, status);
    assert_eq!(failure.correlation_id, None);
    assert_eq!(failure.message, "The request could not be completed");
    assert!(!format!("{failure:?} {failure}").contains(SECRET));
}

fn assert_request(request: &str, path: &str, opted_in: bool, expected: Value) {
    let (headers, body) = request.split_once("\r\n\r\n").unwrap();
    assert!(headers.starts_with(&format!("POST {path} HTTP/1.1\r\n")));
    let lower = headers.to_ascii_lowercase();
    assert!(lower.contains("\r\napi-key: test-api-key"));
    assert_eq!(lower.contains("\r\nx-notbot-protocol: 1"), opted_in);
    if !opted_in {
        assert!(!lower.contains("x-notbot-protocol"));
    }
    assert_eq!(serde_json::from_str::<Value>(body).unwrap(), expected);
}

#[tokio::test]
async fn propagates_all_application_codes_and_statuses_with_safe_text() {
    let codes = [
        "InvalidSession",
        "SessionExpired",
        "ExistingIdentity",
        "PresentationNotFound",
        "MissingCredential",
        "SitePassRecalculationRequired",
        "AlreadyUpgraded",
        "NotNewPerson",
        "UpgradeTooSoon",
        "RecoveryDataNotFound",
        "RequestNotActive",
        "MetadataUnavailable",
        "ProtocolUnsupported",
        "NonceNotFound",
        "UnsupportedPresentation",
        "InvalidInput",
        "ServerError",
        "Database",
        "Gone",
        "IO",
        "Forbidden",
        "Conflict",
        "InvalidProof",
    ];
    for (index, code) in codes.into_iter().enumerate() {
        let status = [400, 401, 403, 404, 409, 410, 422, 429, 500, 503][index % 10];
        // Media types are case insensitive and permit parameters.
        let (client, task) = mock(
            status,
            "Content-Type: Application/Vnd.Notbot.Error+Json; charset=utf-8\r\n",
            &envelope(code).to_string(),
        );
        let failure = client
            .generate_presentation_v1(generate_request())
            .await
            .unwrap_err();
        assert_eq!(failure.version, 1);
        assert_eq!(failure.code, code);
        assert_eq!(failure.status, Some(status));
        assert_eq!(failure.correlation_id.as_deref(), Some(CORRELATION));
        match code {
            "NonceNotFound" => assert_eq!(failure.message, "Device linking code was not found"),
            "UnsupportedPresentation" => {
                assert_eq!(failure.message, "Unsupported presentation version")
            }
            _ => {}
        }
        assert!(!format!("{failure:?} {failure}").contains(SECRET));
        assert_request(
            &task.join().unwrap(),
            "/signature/presentation",
            true,
            serde_json::to_value(generate_request()).unwrap(),
        );
    }
}

#[tokio::test]
async fn verify_uses_same_typed_contract_without_interpreting_message() {
    let mut wire = envelope("MetadataUnavailable");
    wire["message"] = json!("InvalidSession: misleading body text");
    let (client, task) = mock(
        404,
        &format!("Content-Type: {MEDIA}\r\n"),
        &wire.to_string(),
    );
    let failure = client
        .verify_presentation_v1(verify_request())
        .await
        .unwrap_err();
    assert_eq!(failure.code, "MetadataUnavailable");
    assert_eq!(failure.message, "Signature metadata is unavailable");
    assert_eq!(failure.status, Some(404));
    assert_request(
        &task.join().unwrap(),
        "/signature/verify",
        true,
        serde_json::to_value(verify_request()).unwrap(),
    );
}

#[tokio::test]
async fn ignores_additive_fields_and_preserves_future_codes_with_generic_text() {
    for legacy_body_bytes in [Value::Null, json!(1234)] {
        let mut wire = envelope("FutureApplicationFailure");
        wire["legacy_body_bytes"] = legacy_body_bytes;
        wire["future_details"] = json!({"private": SECRET});
        let (client, task) = mock(
            422,
            &format!("Content-Type: {MEDIA}\r\n"),
            &wire.to_string(),
        );
        let failure = client
            .verify_presentation_v1(verify_request())
            .await
            .unwrap_err();
        assert_eq!(failure.version, 1);
        assert_eq!(failure.code, "FutureApplicationFailure");
        assert_eq!(failure.message, "The request could not be completed");
        assert_eq!(failure.status, Some(422));
        assert_eq!(failure.correlation_id.as_deref(), Some(CORRELATION));
        assert!(!format!("{failure:?} {failure}").contains(SECRET));
        task.join().unwrap();
    }
}

#[tokio::test]
async fn invalid_contracts_are_safe_internal_failures() {
    let valid = envelope("InvalidSession");
    let mut cases = vec![
        (format!("Content-Type: {MEDIA}\r\n"), SECRET.to_string()),
        (
            "Content-Type: application/json\r\n".into(),
            valid.to_string(),
        ),
        ("Content-Type: text/html\r\n".into(), SECRET.to_string()),
        (String::new(), valid.to_string()),
        (
            format!("Content-Type: {MEDIA}\r\nx-notbot-protocol: 2\r\n"),
            valid.to_string(),
        ),
        (
            format!("Content-Type: {MEDIA}\r\nx-notbot-protocol: 1\r\nx-notbot-protocol: 1\r\n"),
            valid.to_string(),
        ),
        (
            format!("Content-Type: {MEDIA}\r\nContent-Type: {MEDIA}\r\n"),
            valid.to_string(),
        ),
    ];
    for (field, value) in [
        ("version", json!(2)),
        ("version", json!("1")),
        ("code", json!(SECRET)),
        ("code", json!("invalid_session")),
        ("code", json!("")),
        ("code", json!("Unknown-Code")),
        ("code", json!("A".repeat(65))),
        ("correlation_id", json!(SECRET)),
        ("correlation_id", json!("b8621e5275244cac9991893fbe6bf7a2")),
        ("message", Value::Null),
    ] {
        let mut wire = valid.clone();
        wire[field] = value;
        cases.push((format!("Content-Type: {MEDIA}\r\n"), wire.to_string()));
    }
    for field in ["version", "code", "message", "correlation_id"] {
        let mut wire = valid.clone();
        wire.as_object_mut().unwrap().remove(field);
        cases.push((format!("Content-Type: {MEDIA}\r\n"), wire.to_string()));
    }
    cases.push((format!("Content-Type: {MEDIA}\r\n"), "x".repeat(65537)));
    for (headers, body) in cases {
        let (client, task) = mock(502, &headers, &body);
        assert_internal(
            &client
                .verify_presentation_v1(verify_request())
                .await
                .unwrap_err(),
            Some(502),
        );
        task.join().unwrap();
    }
}

#[tokio::test]
async fn v1_success_payloads_are_unchanged_and_bad_success_is_redacted() {
    let (client, task) = mock(
        200,
        "Content-Type: application/json\r\n",
        r#"{"compressed_presentation":[1,2,3]}"#,
    );
    assert_eq!(
        client
            .generate_presentation_v1(generate_request())
            .await
            .unwrap()
            .compressed_presentation,
        vec![1, 2, 3]
    );
    task.join().unwrap();

    let body = json!({"alias_did":{"launcher_id":vec![0; 32]}, "site_pass":null, "claims":[],
        "timestamp":1, "presentation":[3,2,1], "petname":"name", "lifehash":[], "reserve_names":[]});
    let (client, task) = mock(
        200,
        "Content-Type: application/json\r\nx-notbot-protocol: 1\r\n",
        &body.to_string(),
    );
    assert_eq!(
        client
            .verify_presentation_v1(verify_request())
            .await
            .unwrap()
            .presentation,
        vec![3, 2, 1]
    );
    task.join().unwrap();

    for (headers, body) in [
        (
            "Content-Type: application/json\r\n".to_string(),
            SECRET.to_string(),
        ),
        (
            format!("Content-Type: {MEDIA}\r\n"),
            envelope("InvalidSession").to_string(),
        ),
    ] {
        let (client, task) = mock(200, &headers, &body);
        assert_internal(
            &client
                .generate_presentation_v1(generate_request())
                .await
                .unwrap_err(),
            Some(200),
        );
        task.join().unwrap();
    }
}

#[tokio::test]
async fn transport_and_body_read_failures_are_redacted() {
    let client = SignatureClient::with_url_and_api_key("not-a-url-SECRET", SECRET);
    assert_internal(
        &client
            .generate_presentation_v1(generate_request())
            .await
            .unwrap_err(),
        None,
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let client = SignatureClient::with_url_and_api_key(&url, SECRET);
    assert_internal(
        &client
            .verify_presentation_v1(verify_request())
            .await
            .unwrap_err(),
        None,
    );

    let (client, task) = mock(
        503,
        &format!("Content-Type: {MEDIA}\r\nTransfer-Encoding: chunked\r\n"),
        "invalid chunk",
    );
    assert_internal(
        &client
            .verify_presentation_v1(verify_request())
            .await
            .unwrap_err(),
        Some(503),
    );
    task.join().unwrap();
}

#[tokio::test]
async fn legacy_methods_keep_headerless_requests_and_original_results() {
    let (client, task) = mock(
        200,
        "Content-Type: application/json\r\n",
        r#"{"compressed_presentation":[9]}"#,
    );
    assert_eq!(
        client
            .generate_presentation(generate_request())
            .await
            .unwrap()
            .compressed_presentation,
        vec![9]
    );
    assert_request(
        &task.join().unwrap(),
        "/signature/presentation",
        false,
        serde_json::to_value(generate_request()).unwrap(),
    );

    let (client, task) = mock(
        400,
        "Content-Type: text/plain\r\n",
        "legacy validation failure",
    );
    let failure = client
        .verify_presentation(verify_request())
        .await
        .unwrap_err();
    assert_eq!(failure.code, julia_web_sdk::error::ErrorCode::ServerError);
    assert!(failure.message.contains("legacy validation failure"));
    assert_request(
        &task.join().unwrap(),
        "/signature/verify",
        false,
        serde_json::to_value(verify_request()).unwrap(),
    );
}
