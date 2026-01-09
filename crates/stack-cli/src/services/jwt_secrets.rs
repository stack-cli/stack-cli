use crate::error::Error;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use k8s_openapi::api::core::v1::Secret;
use kube::api::{DeleteParams, Patch, PatchParams};
use kube::{Api, Client};
use rand::{distr::Alphanumeric, Rng};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

pub const JWT_AUTH_SECRET_NAME: &str = "jwt-auth";
pub const JWT_SECRET_KEY: &str = "jwt-secret";
pub const JWT_ANON_TOKEN_KEY: &str = "anon-jwt";
pub const JWT_SERVICE_ROLE_TOKEN_KEY: &str = "service-role-jwt";
const JWT_ISSUER: &str = "stack";
const JWT_EXP_SECONDS: u64 = 60 * 60 * 24 * 365 * 10;

#[derive(Serialize)]
struct JwtClaims {
    role: String,
    iss: String,
    exp: u64,
}

pub async fn ensure_secret(
    client: Client,
    namespace: &str,
) -> Result<(), Error> {
    let secret_api: Api<Secret> = Api::namespaced(client, namespace);
    let existing = secret_api.get(JWT_AUTH_SECRET_NAME).await.ok();

    let jwt_secret = existing
        .as_ref()
        .and_then(|secret| read_secret_field(secret, JWT_SECRET_KEY))
        .unwrap_or_else(random_token);

    let anon_jwt = match existing
        .as_ref()
        .and_then(|secret| read_secret_field(secret, JWT_ANON_TOKEN_KEY))
    {
        Some(value) => value,
        None => build_jwt(&jwt_secret, "anon")?,
    };

    let service_role_jwt = match existing
        .as_ref()
        .and_then(|secret| read_secret_field(secret, JWT_SERVICE_ROLE_TOKEN_KEY))
    {
        Some(value) => value,
        None => build_jwt(&jwt_secret, "service_role")?,
    };

    let secret_manifest = serde_json::json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": JWT_AUTH_SECRET_NAME,
            "namespace": namespace
        },
        "stringData": {
            JWT_SECRET_KEY: jwt_secret,
            JWT_ANON_TOKEN_KEY: anon_jwt,
            JWT_SERVICE_ROLE_TOKEN_KEY: service_role_jwt
        }
    });

    secret_api
        .patch(
            JWT_AUTH_SECRET_NAME,
            &PatchParams::apply(crate::MANAGER).force(),
            &Patch::Apply(secret_manifest),
        )
        .await?;

    Ok(())
}

pub async fn delete(client: Client, namespace: &str) -> Result<(), Error> {
    let secret_api: Api<Secret> = Api::namespaced(client, namespace);
    if secret_api.get(JWT_AUTH_SECRET_NAME).await.is_ok() {
        secret_api
            .delete(JWT_AUTH_SECRET_NAME, &DeleteParams::default())
            .await?;
    }

    Ok(())
}

pub async fn get_token(
    client: Client,
    namespace: &str,
    key: &str,
) -> Result<Option<String>, Error> {
    let secret_api: Api<Secret> = Api::namespaced(client, namespace);
    match secret_api.get(JWT_AUTH_SECRET_NAME).await {
        Ok(secret) => Ok(read_secret_field(&secret, key)),
        Err(kube::Error::Api(err)) if err.code == 404 => Ok(None),
        Err(err) => Err(Error::from(err)),
    }
}

fn build_jwt(secret: &str, role: &str) -> Result<String, Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| Error::Other(err.to_string()))?
        .as_secs();
    let claims = JwtClaims {
        role: role.to_string(),
        iss: JWT_ISSUER.to_string(),
        exp: now + JWT_EXP_SECONDS,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|err: jsonwebtoken::errors::Error| Error::Other(err.to_string()))
}

fn random_token() -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

fn read_secret_field(secret: &Secret, key: &str) -> Option<String> {
    if let Some(data) = &secret.data {
        if let Some(value) = data.get(key) {
            if let Ok(val) = String::from_utf8(value.0.clone()) {
                return Some(val);
            }
        }
    }

    secret
        .string_data
        .as_ref()
        .and_then(|map| map.get(key).cloned())
}
