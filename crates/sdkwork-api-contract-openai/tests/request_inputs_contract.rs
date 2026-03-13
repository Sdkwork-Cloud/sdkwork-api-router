use sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::responses::CreateResponseRequest;

#[test]
fn parses_chat_completion_request_shape() {
    let request: CreateChatCompletionRequest = serde_json::from_str(
        r#"{
            "model":"gpt-4.1",
            "messages":[{"role":"user","content":"hello"}],
            "stream":true
        }"#,
    )
    .unwrap();

    assert_eq!(request.model, "gpt-4.1");
    assert_eq!(request.messages.len(), 1);
    assert_eq!(request.stream, Some(true));
}

#[test]
fn parses_response_request_shape() {
    let request: CreateResponseRequest =
        serde_json::from_str(r#"{"model":"gpt-4.1","input":"hello","stream":false}"#).unwrap();

    assert_eq!(request.model, "gpt-4.1");
    assert_eq!(request.stream, Some(false));
    assert_eq!(request.input, serde_json::Value::String("hello".to_owned()));
}

#[test]
fn parses_embedding_request_shape() {
    let request: CreateEmbeddingRequest =
        serde_json::from_str(r#"{"model":"text-embedding-3-large","input":["hello","world"]}"#)
            .unwrap();

    assert_eq!(request.model, "text-embedding-3-large");
    assert!(request.input.is_array());
}
