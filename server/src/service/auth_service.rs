use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata};
use openidconnect::reqwest::Client as HttpClient;
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tower_sessions::Session;
use tracing::info;
use uuid::Uuid;

use crate::api::error::AppError;
use crate::entities::user;

pub struct AuthService {
    provider_metadata: CoreProviderMetadata,
    client_id: ClientId,
    client_secret: ClientSecret,
    redirect_url: RedirectUrl,
    http_client: HttpClient,
    db: DatabaseConnection,
}

impl AuthService {
    pub async fn new(
        issuer_url: &str,
        client_id: &str,
        client_secret: &str,
        redirect_url: &str,
        db: DatabaseConnection,
    ) -> Result<Self, AppError> {
        let http_client = HttpClient::new();

        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(issuer_url.to_string())
                .map_err(|e| AppError::Internal(format!("Invalid issuer URL: {}", e)))?,
            &http_client,
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to discover provider: {}", e)))?;

        let claims_supported: Vec<&str> = provider_metadata
            .claims_supported()
            .unwrap()
            .iter()
            .map(|x| x.as_str())
            .collect();

        info!(
            "OIDC provider metadata successful, issuer url: {}",
            provider_metadata.issuer()
        );
        info!("     supported claims: {}", claims_supported.join(", "));

        Ok(Self {
            provider_metadata,
            client_id: ClientId::new(client_id.to_string()),
            client_secret: ClientSecret::new(client_secret.to_string()),
            redirect_url: RedirectUrl::new(redirect_url.to_string())
                .map_err(|e| AppError::Internal(format!("Invalid redirect URL: {}", e)))?,
            http_client,
            db,
        })
    }

    pub async fn get_user_from_session(&self, session: &Session) -> Result<user::Model, AppError> {
        let user_id: uuid::Uuid = session
            .get("user_id")
            .await
            .unwrap()
            .ok_or_else(|| AppError::Auth("Not logged in".to_string()))?;

        user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Auth("User not found".to_string()))
    }

    pub fn get_auth_url(&self) -> (String, CsrfToken, Nonce, PkceCodeVerifier) {
        let client = CoreClient::from_provider_metadata(
            self.provider_metadata.clone(),
            self.client_id.clone(),
            Some(self.client_secret.clone()),
        )
        .set_redirect_uri(self.redirect_url.clone());

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token, nonce) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        (auth_url.to_string(), csrf_token, nonce, pkce_verifier)
    }

    pub async fn authenticate(
        &self,
        code: String,
        nonce: Nonce,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<user::Model, AppError> {
        let client = CoreClient::from_provider_metadata(
            self.provider_metadata.clone(),
            self.client_id.clone(),
            Some(self.client_secret.clone()),
        )
        .set_redirect_uri(self.redirect_url.clone());

        let token_response = client
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|e| AppError::Auth(format!("Failed to create exchange request: {}", e)))?
            .set_pkce_verifier(pkce_verifier)
            .request_async(&self.http_client)
            .await
            .map_err(|e| AppError::Auth(format!("Failed to exchange code: {}", e)))?;

        let id_token = token_response
            .id_token()
            .ok_or_else(|| AppError::Auth("No ID token found".to_string()))?;

        let claims = id_token
            .claims(&client.id_token_verifier(), &nonce)
            .map_err(|e| AppError::Auth(format!("Failed to verify ID token: {}", e)))?;

        let email = claims
            .email()
            .ok_or_else(|| AppError::Auth("Email missing from ID token".to_string()))?
            .to_string();

        let username = if let Some(nickname) = claims.nickname() {
            nickname
                .get(None)
                .map(|n| n.to_string())
                .unwrap_or_else(|| email.split('@').next().unwrap_or("user").to_string())
        } else if let Some(preferred_username) = claims.preferred_username() {
            preferred_username.to_string()
        } else {
            email.split('@').next().unwrap_or("user").to_string()
        };

        // Find or create user
        let existing_user = user::Entity::find()
            .filter(user::Column::Email.eq(&email))
            .one(&self.db)
            .await?;

        if let Some(user) = existing_user {
            Ok(user)
        } else {
            let new_user = user::ActiveModel {
                id: Set(Uuid::new_v4()),
                email: Set(email),
                username: Set(username),
            };
            let user = new_user.insert(&self.db).await?;
            Ok(user)
        }
    }
}
