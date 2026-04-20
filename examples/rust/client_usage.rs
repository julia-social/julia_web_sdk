use dg_xch_core::blockchain::sized_bytes::Bytes32;
use dg_xch_core::traits::SizedBytes;
use julia_web_sdk::signature_client::SignatureClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SignatureClient::with_url("https://example.com");

    let request_id = client.get_signature_request_id().await?;
    println!("request id: {request_id}");

    let status = client.get_signature_status().await?;
    println!("signature status: {status}");

    let nonce = Bytes32::new([0u8; 32]);
    let presentation = client
        .generate_signature_presentation(&request_id, nonce)
        .await?;
    println!(
        "presentation bytes: {}",
        presentation.compressed_presentation.len()
    );

    // Placeholder for local testing.
    // In production, pass the signed bytes returned by not.bot.
    client.verify_signature_presentation(&request_id, vec![]).await?;
    println!("verification request submitted");

    Ok(())
}
