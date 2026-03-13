use sdkwork_api_contract_openai::threads::{
    CreateThreadMessageRequest, CreateThreadRequest, DeleteThreadMessageResponse,
    DeleteThreadResponse, ListThreadMessagesResponse, ThreadMessageObject, ThreadObject,
    UpdateThreadMessageRequest, UpdateThreadRequest,
};

#[test]
fn serializes_thread_resource_contracts() {
    let request = CreateThreadRequest::with_metadata(serde_json::json!({"workspace":"default"}));
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["metadata"]["workspace"], "default");

    let thread =
        ThreadObject::with_metadata("thread_1", serde_json::json!({"workspace":"default"}));
    let thread_json = serde_json::to_value(thread).unwrap();
    assert_eq!(thread_json["object"], "thread");
    assert_eq!(thread_json["metadata"]["workspace"], "default");

    let update = UpdateThreadRequest::with_metadata(serde_json::json!({"workspace":"next"}));
    let update_json = serde_json::to_value(update).unwrap();
    assert_eq!(update_json["metadata"]["workspace"], "next");

    let deleted = DeleteThreadResponse::deleted("thread_1");
    let deleted_json = serde_json::to_value(deleted).unwrap();
    assert_eq!(deleted_json["object"], "thread.deleted");
    assert_eq!(deleted_json["deleted"], true);
}

#[test]
fn serializes_thread_message_contracts() {
    let request = CreateThreadMessageRequest::text("user", "hello");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["role"], "user");
    assert_eq!(request_json["content"], "hello");

    let update = UpdateThreadMessageRequest::with_metadata(serde_json::json!({"pinned":"true"}));
    let update_json = serde_json::to_value(update).unwrap();
    assert_eq!(update_json["metadata"]["pinned"], "true");

    let message = ThreadMessageObject::text("msg_1", "thread_1", "assistant", "hello");
    let message_json = serde_json::to_value(message).unwrap();
    assert_eq!(message_json["object"], "thread.message");
    assert_eq!(message_json["thread_id"], "thread_1");
    assert_eq!(message_json["content"][0]["type"], "text");

    let list = ListThreadMessagesResponse::new(vec![ThreadMessageObject::text(
        "msg_1",
        "thread_1",
        "assistant",
        "hello",
    )]);
    let list_json = serde_json::to_value(list).unwrap();
    assert_eq!(list_json["object"], "list");
    assert_eq!(list_json["first_id"], "msg_1");
    assert_eq!(list_json["last_id"], "msg_1");
    assert_eq!(list_json["has_more"], false);

    let deleted = DeleteThreadMessageResponse::deleted("msg_1");
    let deleted_json = serde_json::to_value(deleted).unwrap();
    assert_eq!(deleted_json["object"], "thread.message.deleted");
    assert_eq!(deleted_json["deleted"], true);
}
