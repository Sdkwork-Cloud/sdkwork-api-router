use sdkwork_api_contract_openai::models::{ListModelsResponse, ModelObject};

#[test]
fn wraps_models_in_list_object() {
    let response = ListModelsResponse::new(vec![ModelObject::new("model-1", "sdkwork")]);
    assert_eq!(response.object, "list");
    assert_eq!(response.data.len(), 1);
}
