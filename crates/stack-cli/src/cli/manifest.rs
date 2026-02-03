use crate::operator::crd::StackApp;
use anyhow::{anyhow, Context, Result};
use serde_yaml::{Mapping, Value};
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
        merge_into_spec(spec, profile_map);
        Ok(())
    } else {
        Err(anyhow!("Profile '{}' must be a map", profile_name))
    }
}

fn merge_into_spec(spec: &mut Mapping, overlay: &Mapping) {
    for (key, value) in overlay {
        match spec.get_mut(key) {
            Some(existing) => merge_value(existing, value),
            None => {
                spec.insert(key.clone(), value.clone());
            }
        }
    }
}

fn merge_value(base: &mut Value, overlay: &Value) {
    match overlay {
        Value::Mapping(overlay_map) => {
            if let Value::Mapping(base_map) = base {
                merge_into_spec(base_map, overlay_map);
            } else {
                *base = Value::Mapping(overlay_map.clone());
            }
        }
        _ => {
            *base = overlay.clone();
        }
    }
}
