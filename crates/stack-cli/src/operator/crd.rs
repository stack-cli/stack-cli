use kube::CustomResource;
use schemars::{json_schema, Schema, SchemaGenerator, JsonSchema};
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
    pub services: Services,
    #[serde(default)]
    pub components: Components,
}

/// Services to deploy into the namespace (web and optional helpers).
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[schemars(schema_with = "services_schema")]
pub struct Services {
    pub web: ServiceSpec,
    #[serde(flatten, default)]
    pub extra: std::collections::BTreeMap<String, ServiceSpec>,
}

fn services_schema(_gen: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "object",
        "x-kubernetes-preserve-unknown-fields": true
    })
}

/// Optional platform components.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema, Default)]
pub struct Components {
    pub db: Option<DbConfig>,
    pub oidc: Option<OidcConfig>,
    pub auth: Option<SupabaseAuthConfig>,
    pub storage: Option<StorageConfig>,
    pub cloudflare: Option<CloudflareConfig>,
    pub ingress: Option<IngressConfig>,
    pub realtime: Option<RealtimeConfig>,
    pub rest: Option<RestConfig>,
    pub document_engine: Option<DocumentEngineConfig>,
    pub selenium: Option<SeleniumConfig>,
    pub mailhog: Option<MailhogConfig>,
}

/// User-defined environment variable sourced from plaintext.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}

/// User-defined environment variable sourced from a Kubernetes secret.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct SecretEnvVar {
    pub name: String,
    pub secret_name: String,
    pub secret_key: String,
}

/// Application service container reference.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct ServiceSpec {
    /// Fully-qualified container image reference (e.g. ghcr.io/org/app:tag)
    pub image: String,
    /// Container port exposed by the application (e.g. 7903). Required for services.web.
    pub port: Option<u16>,
    /// Optional list of plaintext environment variables injected into the web pod.
    #[serde(default)]
    pub env: Vec<EnvVar>,
    /// Optional list of secret-backed environment variables injected into the web pod.
    #[serde(default)]
    pub secret_env: Vec<SecretEnvVar>,
    /// Optional init container to run before the main web container starts.
    pub init: Option<WebInit>,
    /// Optional environment variable name to receive the application DATABASE_URL (from `database-urls/application-url`).
    pub database_url: Option<String>,
    /// Optional environment variable name to receive the migrations/superuser URL (from `database-urls/migrations-url`).
    pub migrations_database_url: Option<String>,
    /// Optional environment variable name to receive the readonly URL (from `database-urls/readonly-url`).
    pub readonly_database_url: Option<String>,
}

// Extra services use the same schema as the primary web service.

/// Optional init container configuration for the web service.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct WebInit {
    /// Image to run as init container.
    pub image: String,
    /// Optional list of plaintext environment variables injected into the init container.
    #[serde(default)]
    pub env: Vec<EnvVar>,
    /// Optional list of secret-backed environment variables injected into the init container.
    #[serde(default)]
    pub secret_env: Vec<SecretEnvVar>,
    /// Optional environment variable name to receive the application DATABASE_URL (from `database-urls/application-url`).
    pub database_url: Option<String>,
    /// Optional environment variable name to receive the migrations/superuser URL (from `database-urls/migrations-url`).
    pub migrations_database_url: Option<String>,
    /// Optional environment variable name to receive the readonly URL (from `database-urls/readonly-url`).
    pub readonly_database_url: Option<String>,
}

/// Optional database configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct DbConfig {
    /// Danger: overrides generated DB passwords. Use only for local development.
    pub danger_override_password: Option<String>,
    /// Optional NodePort number to expose the database service.
    pub expose_db_port: Option<u16>,
}

/// Optional OIDC authentication configuration (Keycloak + oauth2-proxy).
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct OidcConfig {
    /// Public hostname that Cloudflare/Keycloak should use for redirects.
    #[serde(rename = "hostname-url")]
    pub hostname_url: Option<String>,
    /// Optional NodePort number to expose the auth (nginx) service.
    pub expose_auth_port: Option<u16>,
    /// When true, allow the Keycloak admin console to be proxied via /oidc/admin.
    pub expose_admin: Option<bool>,
}

/// Optional Supabase Auth (GoTrue) configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct SupabaseAuthConfig {
    /// External URL for GoTrue (e.g. https://example.com/auth).
    pub api_external_url: String,
    /// Site URL for GoTrue (e.g. https://example.com/auth).
    pub site_url: String,
}

/// Optional Supabase storage configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct StorageConfig {
    /// If set, storage will read S3 credentials and settings from this secret instead of the default.
    /// Expected keys: STORAGE_S3_BUCKET, STORAGE_S3_ENDPOINT, STORAGE_S3_REGION, STORAGE_S3_FORCE_PATH_STYLE,
    /// AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, S3_PROTOCOL_ACCESS_KEY_ID, S3_PROTOCOL_ACCESS_KEY_SECRET.
    pub s3_secret_name: Option<String>,
    /// When true, deploy the bundled MinIO instance; defaults to true when no s3_secret_name is provided.
    pub install_minio: Option<bool>,
}

/// Optional ingress configuration for exposing nginx via NodePort.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct IngressConfig {
    /// Optional NodePort number to expose nginx.
    pub port: Option<u16>,
}

/// Optional Cloudflare tunnel configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct CloudflareConfig {
    /// Optional secret name with tunnel configuration; omit for quick tunnels.
    pub secret_name: Option<String>,
}

/// Optional PostgREST configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct RestConfig {
    /// Comma-separated DB schemas that PostgREST exposes.
    pub db_schemas: Option<String>,
    /// Optional NodePort number to expose the PostgREST service.
    pub expose_rest_port: Option<u16>,
    /// Optional JWT expiry to set in app settings (seconds).
    pub jwt_expiry: Option<String>,
}

/// Optional Realtime configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct RealtimeConfig {}

/// Optional document engine configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct DocumentEngineConfig {}

/// Optional Selenium configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct SeleniumConfig {
    /// Optional container image to run.
    pub image: Option<String>,
    /// Optional container port for the WebDriver endpoint.
    pub port: Option<u16>,
    /// Optional container port for VNC.
    pub vnc_port: Option<u16>,
    /// Optional shared memory size (e.g. 2Gi).
    pub shm_size: Option<String>,
    /// Optional NodePort to expose the WebDriver endpoint.
    pub expose_webdriver_port: Option<u16>,
    /// Optional NodePort to expose the VNC endpoint.
    pub expose_vnc_port: Option<u16>,
}

/// Optional MailHog configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct MailhogConfig {
    /// Optional container image to run.
    pub image: Option<String>,
    /// Optional container port for SMTP.
    pub smtp_port: Option<u16>,
    /// Optional container port for the web UI.
    pub web_port: Option<u16>,
    /// Optional NodePort to expose SMTP.
    pub expose_smtp_port: Option<u16>,
    /// Optional NodePort to expose the web UI.
    pub expose_web_port: Option<u16>,
}
