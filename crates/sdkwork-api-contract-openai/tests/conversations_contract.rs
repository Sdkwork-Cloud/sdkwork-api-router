use sdkwork_api_contract_openai::conversations::{
    ConversationItemObject, ConversationObject, CreateConversationItemsRequest,
    CreateConversationRequest, DeleteConversationItemResponse, DeleteConversationResponse,
    ListConversationItemsResponse, ListConversationsResponse, UpdateConversationRequest,
};

#[test]
fn serializes_conversation_contracts() {
    let create_request =
        CreateConversationRequest::with_metadata(serde_json::json!({"workspace":"default"}));
    let create_json = serde_json::to_value(create_request).unwrap();
    assert_eq!(create_json["metadata"]["workspace"], "default");

    let conversation =
        ConversationObject::with_metadata("conv_1", serde_json::json!({"workspace":"default"}));
    let conversation_json = serde_json::to_value(conversation).unwrap();
    assert_eq!(conversation_json["object"], "conversation");
    assert_eq!(conversation_json["metadata"]["workspace"], "default");

    let list = ListConversationsResponse::new(vec![ConversationObject::new("conv_1")]);
    let list_json = serde_json::to_value(list).unwrap();
    assert_eq!(list_json["object"], "list");
    assert_eq!(list_json["data"][0]["id"], "conv_1");

    let update = UpdateConversationRequest::with_metadata(serde_json::json!({"workspace":"next"}));
    let update_json = serde_json::to_value(update).unwrap();
    assert_eq!(update_json["metadata"]["workspace"], "next");

    let deleted = DeleteConversationResponse::deleted("conv_1");
    let deleted_json = serde_json::to_value(deleted).unwrap();
    assert_eq!(deleted_json["object"], "conversation.deleted");
    assert_eq!(deleted_json["deleted"], true);
}

#[test]
fn serializes_conversation_item_contracts() {
    let create_request = CreateConversationItemsRequest::new(vec![serde_json::json!({
        "id":"item_1",
        "type":"message",
        "role":"user",
        "content":[{"type":"input_text","text":"hello"}]
    })]);
    let create_json = serde_json::to_value(create_request).unwrap();
    assert_eq!(create_json["items"][0]["id"], "item_1");

    let item = ConversationItemObject::message("item_1", "assistant", "hello");
    let item_json = serde_json::to_value(item).unwrap();
    assert_eq!(item_json["object"], "conversation.item");
    assert_eq!(item_json["role"], "assistant");

    let list = ListConversationItemsResponse::new(vec![ConversationItemObject::message(
        "item_1",
        "assistant",
        "hello",
    )]);
    let list_json = serde_json::to_value(list).unwrap();
    assert_eq!(list_json["object"], "list");
    assert_eq!(list_json["data"][0]["id"], "item_1");

    let deleted = DeleteConversationItemResponse::deleted("item_1");
    let deleted_json = serde_json::to_value(deleted).unwrap();
    assert_eq!(deleted_json["object"], "conversation.item.deleted");
    assert_eq!(deleted_json["deleted"], true);
}
