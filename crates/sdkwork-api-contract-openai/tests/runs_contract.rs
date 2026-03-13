use sdkwork_api_contract_openai::runs::{
    CreateRunRequest, CreateThreadAndRunRequest, ListRunStepsResponse, ListRunsResponse, RunObject,
    RunStepObject, RunToolOutput, SubmitToolOutputsRunRequest, UpdateRunRequest,
};

#[test]
fn serializes_run_resource_contracts() {
    let request = CreateRunRequest::new("asst_1");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["assistant_id"], "asst_1");

    let create_and_run = CreateThreadAndRunRequest::new(
        "asst_1",
        serde_json::json!({"metadata":{"workspace":"default"}}),
    );
    let create_and_run_json = serde_json::to_value(create_and_run).unwrap();
    assert_eq!(create_and_run_json["assistant_id"], "asst_1");
    assert_eq!(
        create_and_run_json["thread"]["metadata"]["workspace"],
        "default"
    );

    let update = UpdateRunRequest::with_metadata(serde_json::json!({"priority":"high"}));
    let update_json = serde_json::to_value(update).unwrap();
    assert_eq!(update_json["metadata"]["priority"], "high");

    let submit =
        SubmitToolOutputsRunRequest::new(vec![RunToolOutput::new("call_1", "{\"ok\":true}")]);
    let submit_json = serde_json::to_value(submit).unwrap();
    assert_eq!(submit_json["tool_outputs"][0]["tool_call_id"], "call_1");
    assert_eq!(submit_json["tool_outputs"][0]["output"], "{\"ok\":true}");

    let run = RunObject::queued("run_1", "thread_1", "asst_1", "gpt-4.1");
    let run_json = serde_json::to_value(run).unwrap();
    assert_eq!(run_json["object"], "thread.run");
    assert_eq!(run_json["thread_id"], "thread_1");
    assert_eq!(run_json["status"], "queued");

    let list = ListRunsResponse::new(vec![RunObject::queued(
        "run_1", "thread_1", "asst_1", "gpt-4.1",
    )]);
    let list_json = serde_json::to_value(list).unwrap();
    assert_eq!(list_json["object"], "list");
    assert_eq!(list_json["first_id"], "run_1");
    assert_eq!(list_json["last_id"], "run_1");
    assert_eq!(list_json["has_more"], false);
}

#[test]
fn serializes_run_step_contracts() {
    let step = RunStepObject::message_creation("step_1", "thread_1", "run_1", "asst_1", "msg_1");
    let step_json = serde_json::to_value(step).unwrap();
    assert_eq!(step_json["object"], "thread.run.step");
    assert_eq!(step_json["thread_id"], "thread_1");
    assert_eq!(step_json["run_id"], "run_1");
    assert_eq!(step_json["type"], "message_creation");
    assert_eq!(
        step_json["step_details"]["message_creation"]["message_id"],
        "msg_1"
    );

    let list = ListRunStepsResponse::new(vec![RunStepObject::message_creation(
        "step_1", "thread_1", "run_1", "asst_1", "msg_1",
    )]);
    let list_json = serde_json::to_value(list).unwrap();
    assert_eq!(list_json["object"], "list");
    assert_eq!(list_json["first_id"], "step_1");
    assert_eq!(list_json["last_id"], "step_1");
    assert_eq!(list_json["has_more"], false);
}
