use super::super::{unix_timestamp_ms, MarketingGovernanceError};
use super::actionability::{build_campaign_budget_actionability, build_campaign_budget_detail};
use super::audit::{
    build_campaign_budget_lifecycle_audit_record, persist_campaign_budget_lifecycle_audit_record,
};
use super::lookup::load_campaign_budget_context;
use super::types::CampaignBudgetMutationResult;
use sdkwork_api_domain_marketing::{
    CampaignBudgetLifecycleAction, CampaignBudgetLifecycleAuditOutcome, CampaignBudgetStatus,
};
use sdkwork_api_storage_core::AdminStore;

pub async fn mutate_marketing_campaign_budget_lifecycle(
    store: &dyn AdminStore,
    campaign_budget_id: &str,
    action: CampaignBudgetLifecycleAction,
    operator_id: &str,
    request_id: &str,
    reason: &str,
) -> Result<CampaignBudgetMutationResult, MarketingGovernanceError> {
    let now_ms = unix_timestamp_ms();
    let (budget, campaign) = load_campaign_budget_context(store, campaign_budget_id).await?;
    let actionability = build_campaign_budget_actionability(&budget, &campaign);
    let decision = match action {
        CampaignBudgetLifecycleAction::Activate => &actionability.activate,
        CampaignBudgetLifecycleAction::Close => &actionability.close,
    };
    if !decision.allowed {
        let audit = build_campaign_budget_lifecycle_audit_record(
            &budget,
            None,
            action,
            CampaignBudgetLifecycleAuditOutcome::Rejected,
            operator_id,
            request_id,
            reason,
            now_ms,
            decision.reasons.clone(),
        );
        persist_campaign_budget_lifecycle_audit_record(store, &audit).await?;
        return Err(MarketingGovernanceError::invalid_input(
            decision
                .reasons
                .first()
                .cloned()
                .unwrap_or_else(|| "campaign budget lifecycle action is not allowed".to_owned()),
        ));
    }

    let next_status = match action {
        CampaignBudgetLifecycleAction::Activate => CampaignBudgetStatus::Active,
        CampaignBudgetLifecycleAction::Close => CampaignBudgetStatus::Closed,
    };
    let updated_budget = budget
        .clone()
        .with_status(next_status)
        .with_updated_at_ms(now_ms);
    let updated_budget = store
        .insert_campaign_budget_record(&updated_budget)
        .await
        .map_err(MarketingGovernanceError::storage)?;

    let detail = build_campaign_budget_detail(updated_budget.clone(), campaign);
    let audit = build_campaign_budget_lifecycle_audit_record(
        &budget,
        Some(&updated_budget),
        action,
        CampaignBudgetLifecycleAuditOutcome::Applied,
        operator_id,
        request_id,
        reason,
        now_ms,
        Vec::new(),
    );
    let audit = persist_campaign_budget_lifecycle_audit_record(store, &audit).await?;
    Ok(CampaignBudgetMutationResult { detail, audit })
}
