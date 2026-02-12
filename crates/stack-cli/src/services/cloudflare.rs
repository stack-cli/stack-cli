use crate::cli::apply;
use crate::error::Error;
use crate::services::nginx::{NGINX_NAME, NGINX_PORT};
use k8s_openapi::api::core::v1::Secret;
use kube::{Api, Client};

const CLOUDFLARE_QUICK_YAML: &str = r#"---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cloudflared
spec:
  selector:
    matchLabels:
      app: cloudflared
  replicas: 1
  template:
    metadata:
      labels:
        app: cloudflared
    spec:
      containers:
      - name: cloudflared
        image: cloudflare/cloudflared:latest
        args:
        - tunnel
        - --no-autoupdate
        - --protocol
        - http2
        - --url
        - $TARGET_URL
"#;

const CLOUDFLARE_CONFIG_YAML: &str = r#"---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cloudflared
spec:
  selector:
    matchLabels:
      app: cloudflared
  replicas: 1
  template:
    metadata:
      labels:
        app: cloudflared
    spec:
      containers:
      - name: cloudflared
        image: cloudflare/cloudflared:latest
        env:
        - name: TUNNEL_TOKEN
          valueFrom:
            secretKeyRef:
              name: $SECRET_NAME
              key: token
        args:
        - tunnel
        - --config
        - /etc/cloudflared/config/config.yaml
        - run
        volumeMounts:
        - name: config
          mountPath: /etc/cloudflared/config
          readOnly: true
      volumes:
      - name: config
        configMap:
          name: cloudflared
          items:
          - key: config.yaml
            path: config.yaml
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: cloudflared
data:
  config.yaml: |
    tunnel: $TUNNEL_NAME
    ingress:
    - hostname: "*"
      service: $INGRESS_TARGET
"#;

const CLOUDFLARE_CONFIG_NO_TUNNEL_YAML: &str = r#"---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cloudflared
spec:
  selector:
    matchLabels:
      app: cloudflared
  replicas: 1
  template:
    metadata:
      labels:
        app: cloudflared
    spec:
      containers:
      - name: cloudflared
        image: cloudflare/cloudflared:latest
        env:
        - name: TUNNEL_TOKEN
          valueFrom:
            secretKeyRef:
              name: $SECRET_NAME
              key: token
        args:
        - tunnel
        - --config
        - /etc/cloudflared/config/config.yaml
        - run
        volumeMounts:
        - name: config
          mountPath: /etc/cloudflared/config
          readOnly: true
      volumes:
      - name: config
        configMap:
          name: cloudflared
          items:
          - key: config.yaml
            path: config.yaml
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: cloudflared
data:
  config.yaml: |
    ingress:
    - hostname: "*"
      service: $INGRESS_TARGET
"#;

pub const SECRET_TOKEN_KEY: &str = "token";
pub const SECRET_TUNNEL_NAME_KEY: &str = "tunnel_name";
pub const SECRET_INGRESS_TARGET_KEY: &str = "ingress_target";

pub async fn deploy(
    client: &Client,
    namespace: &str,
    secret_name: Option<&str>,
) -> Result<(), Error> {
    let nginx_target = format!(
        "http://{nginx}.{namespace}.svc.cluster.local:{port}",
        nginx = NGINX_NAME,
        namespace = namespace,
        port = NGINX_PORT
    );

    if let Some(secret_name) = secret_name {
        let secret_api: Api<Secret> = Api::namespaced(client.clone(), namespace);
        let secret = secret_api.get(secret_name).await?;
        let tunnel_name = read_secret_field(&secret, SECRET_TUNNEL_NAME_KEY);
        let ingress_target = read_secret_field(&secret, SECRET_INGRESS_TARGET_KEY)
            .unwrap_or_else(|| nginx_target.clone());

        if read_secret_field(&secret, SECRET_TOKEN_KEY).is_none() {
            return Err(Error::Other("Cloudflare secret missing token".to_string()));
        }

        let yaml = if let Some(tunnel_name) = tunnel_name {
            CLOUDFLARE_CONFIG_YAML
                .replace("$SECRET_NAME", secret_name)
                .replace("$TUNNEL_NAME", &tunnel_name)
                .replace("$INGRESS_TARGET", &ingress_target)
        } else {
            CLOUDFLARE_CONFIG_NO_TUNNEL_YAML
                .replace("$SECRET_NAME", secret_name)
                .replace("$INGRESS_TARGET", &ingress_target)
        };
        apply::apply(client, &yaml, Some(namespace))
            .await
            .map_err(Error::from)
    } else {
        let yaml = CLOUDFLARE_QUICK_YAML.replace("$TARGET_URL", &nginx_target);
        apply::apply(client, &yaml, Some(namespace))
            .await
            .map_err(Error::from)
    }
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
