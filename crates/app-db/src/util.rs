pub(crate) fn escape_json(object: &mut serde_json::Value) {
    match object {
        serde_json::Value::Object(ref mut value) => {
            for (_key, val) in value {
                escape_json(val);
            }
        }
        serde_json::Value::Array(ref mut values) => {
            for element in values.iter_mut() {
                escape_json(element)
            }
        }
        serde_json::Value::String(ref mut value) => *value = value.escape_default().to_string(),
        _ => {}
    }
}
