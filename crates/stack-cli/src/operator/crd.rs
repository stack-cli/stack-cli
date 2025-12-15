use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Stack application custom resource specification.
#[derive(CustomResource, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[kube(
    group = "stack-cli.dev",
    version = "v1",
    kind = "StackApp",
    plural = "stackapps",
    derive = "PartialEq",
    namespaced
)]
pub struct StackAppSpec {
    pub web: WebContainer,
    pub auth: Option<AuthConfig>,
    pub db: Option<DbConfig>,
    pub storage: Option<StorageConfig>,
}

/// Web application container reference.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct WebContainer {
    /// Fully-qualified container image reference (e.g. ghcr.io/org/app:tag)
    pub image: String,
    /// Container port exposed by the application (e.g. 7903)
    pub port: u16,
    /// Optional NodePort number to expose the app (nginx) service.
    pub expose_app_port: Option<u16>,
}

/// Optional database configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct DbConfig {
    /// Danger: overrides generated DB passwords. Use only for local development.
    pub danger_insecure_password: Option<String>,
    /// Optional NodePort number to expose the database service.
    pub expose_db_port: Option<u16>,
}

/// Optional authentication configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct AuthConfig {
    /// Public hostname that Cloudflare/Keycloak should use for redirects.
    #[serde(rename = "hostname-url")]
    pub hostname_url: Option<String>,
    /// Static JWT token forwarded by nginx when OIDC is disabled.
    pub jwt: Option<String>,
}

/// Optional Supabase storage configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct StorageConfig {
    /// Optional NodePort number to expose the storage service.
    pub expose_storage_port: Option<u16>,
    /// If set, storage will read S3 credentials and settings from this secret instead of the default.
    /// Expected keys: STORAGE_S3_BUCKET, STORAGE_S3_ENDPOINT, STORAGE_S3_REGION, STORAGE_S3_FORCE_PATH_STYLE,
    /// AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, S3_PROTOCOL_ACCESS_KEY_ID, S3_PROTOCOL_ACCESS_KEY_SECRET.
    pub s3_secret_name: Option<String>,
    /// When true, deploy the bundled MinIO instance; defaults to true when no s3_secret_name is provided.
    pub install_minio: Option<bool>,
}
