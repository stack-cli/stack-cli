use crate::operator::crd::StackApp;
use anyhow::{anyhow, Context, Result};
use serde_yaml::{Mapping, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn load_stackapp(path: &Path, profile: Option<&str>) -> Result<(StackApp, String)> {
    let manifest_raw = fs::read_to_string(path)
        .with_context(|| format!("Failed to read manifest at {}", path.display()))?;

    let mut doc: Value =
        serde_yaml::from_str(&manifest_raw).context("Failed to parse StackApp manifest")?;

    apply_profile(&mut doc, profile)?;

    let merged_yaml =
        serde_yaml::to_string(&doc).context("Failed to serialize StackApp manifest")?;
    let stack_app: StackApp =
        serde_yaml::from_str(&merged_yaml).context("Failed to parse StackApp manifest")?;

    if stack_app.spec.services.web.port.is_none() {
        return Err(anyhow!(
            "spec.services.web.port is required for the web service"
        ));
    }

    Ok((stack_app, merged_yaml))
}

fn apply_profile(doc: &mut Value, profile: Option<&str>) -> Result<()> {
    let spec = doc
        .get_mut("spec")
        .and_then(Value::as_mapping_mut)
        .ok_or_else(|| anyhow!("StackApp manifest is missing spec"))?;

    let profiles_key = Value::String("profiles".to_string());
    let profiles_value = spec.remove(&profiles_key);

    let Some(profile_name) = profile else {
        return Ok(());
    };

    let Some(profiles_value) = profiles_value else {
        return Err(anyhow!("Profile '{}' not found in manifest", profile_name));
    };

    let profiles = profiles_value
        .as_mapping()
        .ok_or_else(|| anyhow!("spec.profiles must be a map"))?;
    let profile_value = profiles
        .get(&Value::String(profile_name.to_string()))
        .ok_or_else(|| anyhow!("Profile '{}' not found in manifest", profile_name))?;

    if let Value::Mapping(profile_map) = profile_value {
        merge_mapping(spec, profile_map, &[]);
        Ok(())
    } else {
        Err(anyhow!("Profile '{}' must be a map", profile_name))
    }
}

fn merge_mapping(spec: &mut Mapping, overlay: &Mapping, path: &[String]) {
    merge_service_env_lists(spec, overlay, path);

    for (key, value) in overlay {
        let Some(key_name) = key.as_str() else {
            continue;
        };

        if is_env_key(key_name) && is_service_env_scope(path) {
            continue;
        }

        let mut child_path = path.to_vec();
        child_path.push(key_name.to_string());

        match spec.get_mut(key) {
            Some(existing) => merge_value(existing, value, &child_path),
            None => {
                spec.insert(key.clone(), value.clone());
            }
        }
    }
}

fn merge_value(base: &mut Value, overlay: &Value, path: &[String]) {
    match overlay {
        Value::Mapping(overlay_map) => {
            if let Value::Mapping(base_map) = base {
                merge_mapping(base_map, overlay_map, path);
            } else {
                *base = Value::Mapping(overlay_map.clone());
            }
        }
        _ => {
            *base = overlay.clone();
        }
    }
}

fn merge_service_env_lists(spec: &mut Mapping, overlay: &Mapping, path: &[String]) {
    if !is_service_env_scope(path) {
        return;
    }

    let overlay_env = get_sequence(overlay, "env");
    let overlay_secret_env = get_sequence(overlay, "secret_env");
    if overlay_env.is_none() && overlay_secret_env.is_none() {
        return;
    }

    let overlay_names = collect_names(overlay_env, overlay_secret_env);

    let merged_env = merged_sequence(get_sequence(spec, "env"), overlay_env, &overlay_names);
    let merged_secret_env = merged_sequence(
        get_sequence(spec, "secret_env"),
        overlay_secret_env,
        &overlay_names,
    );

    spec.insert(
        Value::String("env".to_string()),
        Value::Sequence(merged_env),
    );
    spec.insert(
        Value::String("secret_env".to_string()),
        Value::Sequence(merged_secret_env),
    );
}

fn merged_sequence(
    base: Option<&Vec<Value>>,
    overlay: Option<&Vec<Value>>,
    overlay_names: &HashSet<String>,
) -> Vec<Value> {
    let mut merged: Vec<Value> = base
        .into_iter()
        .flatten()
        .filter(|item| {
            item_name(item)
                .map(|name| !overlay_names.contains(name))
                .unwrap_or(true)
        })
        .cloned()
        .collect();

    if let Some(overlay) = overlay {
        let mut seen = HashSet::new();
        let mut overlay_values = Vec::new();

        for item in overlay.iter().rev() {
            match item_name(item) {
                Some(name) if seen.insert(name.to_string()) => overlay_values.push(item.clone()),
                Some(_) => {}
                None => overlay_values.push(item.clone()),
            }
        }

        overlay_values.reverse();
        merged.extend(overlay_values);
    }

    merged
}

fn collect_names(env: Option<&Vec<Value>>, secret_env: Option<&Vec<Value>>) -> HashSet<String> {
    env.into_iter()
        .flatten()
        .chain(secret_env.into_iter().flatten())
        .filter_map(item_name)
        .map(str::to_string)
        .collect()
}

fn get_sequence<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a Vec<Value>> {
    mapping
        .get(Value::String(key.to_string()))
        .and_then(Value::as_sequence)
}

fn item_name(item: &Value) -> Option<&str> {
    item.as_mapping()
        .and_then(|mapping| mapping.get(Value::String("name".to_string())))
        .and_then(Value::as_str)
}

fn is_env_key(key: &str) -> bool {
    matches!(key, "env" | "secret_env")
}

fn is_service_env_scope(path: &[String]) -> bool {
    matches!(path, [services, _] if services == "services")
        || matches!(path, [services, _, init] if services == "services" && init == "init")
}

#[cfg(test)]
mod tests {
    use super::load_stackapp;
    use crate::operator::crd::{EnvVar, SecretEnvVar};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn profile_env_merges_and_overrides_by_name() {
        let manifest = r#"
apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: test
  namespace: test
spec:
  profiles:
    uat:
      services:
        web:
          env:
            - name: FOO
              value: profile
            - name: BAZ
              value: added
          secret_env:
            - name: SECRET
              secret_name: app
              secret_key: profile-secret
  services:
    web:
      image: app:latest
      port: 8080
      env:
        - name: FOO
          value: base
        - name: BAR
          value: keep
      secret_env:
        - name: SECRET
          secret_name: app
          secret_key: base-secret
"#;

        let path = write_manifest("profile-env-merge", manifest);
        let (stack_app, _) = load_stackapp(&path, Some("uat")).unwrap();

        assert_eq!(
            stack_app.spec.services.web.env,
            vec![
                EnvVar {
                    name: "BAR".to_string(),
                    value: "keep".to_string(),
                },
                EnvVar {
                    name: "FOO".to_string(),
                    value: "profile".to_string(),
                },
                EnvVar {
                    name: "BAZ".to_string(),
                    value: "added".to_string(),
                },
            ]
        );
        assert_eq!(
            stack_app.spec.services.web.secret_env,
            vec![SecretEnvVar {
                name: "SECRET".to_string(),
                secret_name: "app".to_string(),
                secret_key: "profile-secret".to_string(),
            }]
        );
    }

    #[test]
    fn profile_secret_env_overrides_base_plain_env() {
        let manifest = r#"
apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: test
  namespace: test
spec:
  profiles:
    prod:
      services:
        web:
          secret_env:
            - name: SECRET_KEY
              secret_name: app
              secret_key: secret-key
  services:
    web:
      image: app:latest
      port: 8080
      env:
        - name: SECRET_KEY
          value: plain
"#;

        let path = write_manifest("profile-secret-overrides-plain", manifest);
        let (stack_app, _) = load_stackapp(&path, Some("prod")).unwrap();

        assert!(stack_app.spec.services.web.env.is_empty());
        assert_eq!(
            stack_app.spec.services.web.secret_env,
            vec![SecretEnvVar {
                name: "SECRET_KEY".to_string(),
                secret_name: "app".to_string(),
                secret_key: "secret-key".to_string(),
            }]
        );
    }

    #[test]
    fn profile_plain_env_overrides_base_secret_env_in_init() {
        let manifest = r#"
apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: test
  namespace: test
spec:
  profiles:
    prod:
      services:
        web:
          init:
            env:
              - name: DATABASE_URL
                value: postgres://override
  services:
    web:
      image: app:latest
      port: 8080
      init:
        image: init:latest
        secret_env:
          - name: DATABASE_URL
            secret_name: app
            secret_key: database-url
"#;

        let path = write_manifest("profile-plain-overrides-secret-init", manifest);
        let (stack_app, _) = load_stackapp(&path, Some("prod")).unwrap();
        let init = stack_app.spec.services.web.init.unwrap();

        assert_eq!(
            init.env,
            vec![EnvVar {
                name: "DATABASE_URL".to_string(),
                value: "postgres://override".to_string(),
            }]
        );
        assert!(init.secret_env.is_empty());
    }

    #[test]
    fn unrelated_sequences_still_replace() {
        let manifest = r#"
apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: test
  namespace: test
spec:
  profiles:
    prod:
      services:
        web:
          init:
            command:
              - profile
  services:
    web:
      image: app:latest
      port: 8080
      init:
        image: init:latest
        command:
          - base
"#;

        let path = write_manifest("unrelated-sequence-replace", manifest);
        let (_, merged_yaml) = load_stackapp(&path, Some("prod")).unwrap();
        let merged_doc: serde_yaml::Value = serde_yaml::from_str(&merged_yaml).unwrap();
        let command = &merged_doc["spec"]["services"]["web"]["init"]["command"];

        assert_eq!(
            command,
            &serde_yaml::Value::Sequence(vec![serde_yaml::Value::String("profile".to_string())])
        );
    }

    fn write_manifest(name: &str, contents: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{name}-{unique}.yaml"));
        fs::write(&path, contents).unwrap();
        path
    }
}
