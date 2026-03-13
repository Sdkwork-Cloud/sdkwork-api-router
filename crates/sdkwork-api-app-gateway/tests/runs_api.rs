use sdkwork_api_app_gateway::{
    cancel_thread_run, create_thread_and_run, create_thread_run, get_thread_run,
    get_thread_run_step, list_thread_run_steps, list_thread_runs, submit_thread_run_tool_outputs,
    update_thread_run,
};

#[test]
fn creates_and_lists_thread_runs() {
    let created = create_thread_run(
        "tenant-1",
        "project-1",
        "thread_1",
        "asst_1",
        Some("gpt-4.1"),
    )
    .unwrap();
    assert_eq!(created.object, "thread.run");
    assert_eq!(created.thread_id, "thread_1");
    assert_eq!(created.assistant_id.as_deref(), Some("asst_1"));

    let list = list_thread_runs("tenant-1", "project-1", "thread_1").unwrap();
    assert_eq!(list.object, "list");
    assert_eq!(list.data[0].object, "thread.run");
}

#[test]
fn retrieves_updates_cancels_and_submits_tool_outputs_for_runs() {
    let retrieved = get_thread_run("tenant-1", "project-1", "thread_1", "run_1").unwrap();
    assert_eq!(retrieved.id, "run_1");
    assert_eq!(retrieved.thread_id, "thread_1");

    let updated = update_thread_run("tenant-1", "project-1", "thread_1", "run_1").unwrap();
    assert_eq!(updated.id, "run_1");

    let submitted = submit_thread_run_tool_outputs(
        "tenant-1",
        "project-1",
        "thread_1",
        "run_1",
        vec![("call_1", "{\"ok\":true}")],
    )
    .unwrap();
    assert_eq!(submitted.id, "run_1");

    let cancelled = cancel_thread_run("tenant-1", "project-1", "thread_1", "run_1").unwrap();
    assert_eq!(cancelled.id, "run_1");
    assert_eq!(cancelled.status, "cancelled");
}

#[test]
fn creates_thread_and_run_and_lists_steps() {
    let created = create_thread_and_run("tenant-1", "project-1", "asst_1").unwrap();
    assert_eq!(created.object, "thread.run");
    assert_eq!(created.assistant_id.as_deref(), Some("asst_1"));

    let list = list_thread_run_steps("tenant-1", "project-1", "thread_1", "run_1").unwrap();
    assert_eq!(list.object, "list");
    assert_eq!(list.data[0].object, "thread.run.step");

    let retrieved =
        get_thread_run_step("tenant-1", "project-1", "thread_1", "run_1", "step_1").unwrap();
    assert_eq!(retrieved.id, "step_1");
    assert_eq!(retrieved.run_id, "run_1");
}
