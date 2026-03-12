use sdkwork_api_contract_openai::errors::OpenAiErrorResponse;

#[test]
fn serializes_openai_error_shape() {
    let json = serde_json::to_value(OpenAiErrorResponse::new(
        "bad request",
        "invalid_request_error",
    ))
    .unwrap();

    assert_eq!(json["error"]["message"], "bad request");
    assert_eq!(json["error"]["type"], "invalid_request_error");
}
