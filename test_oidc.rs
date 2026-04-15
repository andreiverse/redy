use openidconnect::reqwest::Client as HttpClient;
use openidconnect::{IssuerUrl, core::CoreProviderMetadata};

#[tokio::main]
async fn main() {
    let issuer_url = "https://auth.andrei.vip/application/o/localhost/";
    let http_client = HttpClient::new();
    
    let result = CoreProviderMetadata::discover_async(
        IssuerUrl::new(issuer_url.to_string()).unwrap(),
        &http_client,
    ).await;
    
    match result {
        Ok(_) => println!("Discovery successful!"),
        Err(e) => println!("Discovery failed: {:?}", e),
    }
}
