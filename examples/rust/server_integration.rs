use julia_web_sdk::ServiceBuilder;
use julia_web_sdk::claims::ClaimProperties;

fn build_signature_service() {
    let _service_group = ServiceBuilder::new()
        .request_claims(vec![
            ClaimProperties::Notbot0,
            ClaimProperties::SitePass,
            ClaimProperties::FirstName,
            ClaimProperties::AgeOver18,
        ])
        .require_site_pass(true)
        .message_generator(Box::new(|| {
            "Verifying My Identity with example.com".to_string()
        }))
        .on_success(Box::new(|response, session| {
            Box::pin(async move {
                session.write().await.data.insert(response);
                Ok(())
            })
        }))
        .on_failure(Box::new(|error| {
            let message = error.to_string();
            Box::pin(async move {
                eprintln!("verification failure: {message}");
                Ok(())
            })
        }))
        .expire_time(3600)
        .build();
}
