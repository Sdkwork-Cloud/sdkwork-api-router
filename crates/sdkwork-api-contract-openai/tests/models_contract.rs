use sdkwork_api_contract_openai::models::{DeleteModelResponse, ListModelsResponse, ModelObject};

#[test]
fn wraps_models_in_list_object() {
    let response = ListModelsResponse::new(vec![ModelObject::new("model-1", "sdkwork")]);
    assert_eq!(response.object, "list");
    assert_eq!(response.data.len(), 1);
}

#[test]
fn serializes_model_delete_shape() {
    let json = serde_json::to_value(DeleteModelResponse::deleted("ft:gpt-4.1:sdkwork")).unwrap();
    assert_eq!(json["id"], "ft:gpt-4.1:sdkwork");
    assert_eq!(json["object"], "model");
    assert_eq!(json["deleted"], true);
}
