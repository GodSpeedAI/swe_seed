use serde_json::{Map, Value};

pub const MARKER_BEGIN: &str = "BEGIN SWE_SEED MANAGED";
pub const MARKER_END: &str = "END SWE_SEED MANAGED";

pub fn block(stable_id: &str, body: &str) -> String {
    format!("# {MARKER_BEGIN}: {stable_id}\n{body}\n# {MARKER_END}: {stable_id}\n")
}

pub fn merge_json(existing: Option<&[u8]>, projected: &str) -> anyhow::Result<Vec<u8>> {
    let mut projected_value: Value = serde_json::from_str(projected)?;
    stamp_managed_keys(&mut projected_value);
    let Some(existing_bytes) = existing else {
        return Ok(format!("{}\n", serde_json::to_string_pretty(&projected_value)?).into_bytes());
    };
    let Ok(mut existing_value) = serde_json::from_slice::<Value>(existing_bytes) else {
        return Ok(format!("{}\n", serde_json::to_string_pretty(&projected_value)?).into_bytes());
    };
    let (Some(existing_obj), Some(projected_obj)) =
        (existing_value.as_object_mut(), projected_value.as_object())
    else {
        return Ok(format!("{}\n", serde_json::to_string_pretty(&projected_value)?).into_bytes());
    };
    merge_object(existing_obj, projected_obj);
    Ok(format!("{}\n", serde_json::to_string_pretty(&existing_value)?).into_bytes())
}

fn merge_object(existing: &mut Map<String, Value>, projected: &Map<String, Value>) {
    let current_keys = managed_keys(projected);
    let previous_keys = existing
        .get("swe_seed_managed")
        .and_then(|value| value.get("managed_keys"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    for key in previous_keys {
        if !current_keys.iter().any(|current| current == &key) {
            existing.remove(&key);
        }
    }
    for (key, value) in projected {
        existing.insert(key.clone(), value.clone());
    }
}

fn stamp_managed_keys(value: &mut Value) {
    let keys = value.as_object().map(managed_keys).unwrap_or_default();
    let Some(object) = value.as_object_mut() else {
        return;
    };
    let managed = object
        .entry("swe_seed_managed")
        .or_insert_with(|| Value::Object(Map::new()));
    let Some(managed_object) = managed.as_object_mut() else {
        return;
    };
    managed_object.insert(
        "managed_keys".into(),
        Value::Array(keys.into_iter().map(Value::String).collect()),
    );
}

fn managed_keys(object: &Map<String, Value>) -> Vec<String> {
    let mut keys = object
        .keys()
        .filter(|key| key.as_str() != "swe_seed_managed")
        .cloned()
        .collect::<Vec<_>>();
    keys.sort();
    keys
}
