use sdkwork_api_app_gateway::create_fine_tuning_job;

#[test]
fn returns_fine_tuning_job_object() {
    let response = create_fine_tuning_job("tenant-1", "project-1", "gpt-4.1-mini").unwrap();
    assert_eq!(response.object, "fine_tuning.job");
    assert_eq!(response.model, "gpt-4.1-mini");
}

#[test]
fn lists_fine_tuning_jobs() {
    let response = sdkwork_api_app_gateway::list_fine_tuning_jobs("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].object, "fine_tuning.job");
}

#[test]
fn retrieves_fine_tuning_job() {
    let response =
        sdkwork_api_app_gateway::get_fine_tuning_job("tenant-1", "project-1", "ftjob_1").unwrap();
    assert_eq!(response.id, "ftjob_1");
    assert_eq!(response.object, "fine_tuning.job");
}

#[test]
fn cancels_fine_tuning_job() {
    let response =
        sdkwork_api_app_gateway::cancel_fine_tuning_job("tenant-1", "project-1", "ftjob_1")
            .unwrap();
    assert_eq!(response.id, "ftjob_1");
    assert_eq!(response.status, "cancelled");
}
