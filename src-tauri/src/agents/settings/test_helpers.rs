use prelay_protocol::CatalogLanguageModelResponse;

pub(super) fn catalog_model(id: &str, display_name: &str) -> CatalogLanguageModelResponse {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "display_name": display_name,
    }))
    .unwrap()
}

pub(crate) fn assert_no_null_values(value: &serde_json::Value) {
    match value {
        serde_json::Value::Null => panic!("models.json contains a null value"),
        serde_json::Value::Array(values) => {
            for value in values {
                assert_no_null_values(value);
            }
        }
        serde_json::Value::Object(values) => {
            for value in values.values() {
                assert_no_null_values(value);
            }
        }
        serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => {}
    }
}
