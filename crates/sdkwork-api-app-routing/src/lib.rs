use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{ensure, Result};
use sdkwork_api_app_extension::{
    list_extension_runtime_statuses, matching_runtime_statuses_for_provider,
    ExtensionRuntimeStatusRecord,
};
use sdkwork_api_domain_catalog::ProxyProvider;
use sdkwork_api_domain_routing::{
    select_policy, CompiledRoutingSnapshotRecord, ProjectRoutingPreferences,
    ProviderHealthSnapshot, RoutingCandidateAssessment, RoutingCandidateHealth, RoutingDecision,
    RoutingDecisionLog, RoutingDecisionSource, RoutingPolicy, RoutingProfileRecord,
    RoutingStrategy,
};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance};
use sdkwork_api_policy_routing::{
    builtin_routing_strategy_registry, RoutingStrategyExecutionInput,
    RoutingStrategyExecutionResult,
};
use sdkwork_api_storage_core::AdminStore;
use serde_json::Value;

const DEFAULT_WEIGHT: u64 = 100;
const STRATEGY_STATIC_FALLBACK: &str = "static_fallback";

pub fn service_name() -> &'static str {
    "routing-service"
}

pub struct CreateRoutingPolicyInput<'a> {
    pub policy_id: &'a str,
    pub capability: &'a str,
    pub model_pattern: &'a str,
    pub enabled: bool,
    pub priority: i32,
    pub strategy: Option<RoutingStrategy>,
    pub ordered_provider_ids: &'a [String],
    pub default_provider_id: Option<&'a str>,
    pub max_cost: Option<f64>,
    pub max_latency_ms: Option<u64>,
    pub require_healthy: bool,
}

pub struct CreateRoutingProfileInput<'a> {
    pub profile_id: &'a str,
    pub tenant_id: &'a str,
    pub project_id: &'a str,
    pub name: &'a str,
    pub slug: &'a str,
    pub description: Option<&'a str>,
    pub active: bool,
    pub strategy: Option<RoutingStrategy>,
    pub ordered_provider_ids: &'a [String],
    pub default_provider_id: Option<&'a str>,
    pub max_cost: Option<f64>,
    pub max_latency_ms: Option<u64>,
    pub require_healthy: bool,
    pub preferred_region: Option<&'a str>,
}

#[derive(Debug, Clone, Copy)]
pub struct RouteSelectionContext<'a> {
    pub decision_source: RoutingDecisionSource,
    pub tenant_id: Option<&'a str>,
    pub project_id: Option<&'a str>,
    pub api_key_group_id: Option<&'a str>,
    pub requested_region: Option<&'a str>,
    pub selection_seed: Option<u64>,
}

impl<'a> RouteSelectionContext<'a> {
    pub fn new(decision_source: RoutingDecisionSource) -> Self {
        Self {
            decision_source,
            tenant_id: None,
            project_id: None,
            api_key_group_id: None,
            requested_region: None,
            selection_seed: None,
        }
    }

    pub fn with_tenant_id_option(mut self, tenant_id: Option<&'a str>) -> Self {
        self.tenant_id = tenant_id;
        self
    }

    pub fn with_project_id_option(mut self, project_id: Option<&'a str>) -> Self {
        self.project_id = project_id;
        self
    }

    pub fn with_api_key_group_id_option(mut self, api_key_group_id: Option<&'a str>) -> Self {
        self.api_key_group_id = api_key_group_id;
        self
    }

    pub fn with_requested_region_option(mut self, requested_region: Option<&'a str>) -> Self {
        self.requested_region = requested_region;
        self
    }

    pub fn with_selection_seed_option(mut self, selection_seed: Option<u64>) -> Self {
        self.selection_seed = selection_seed;
        self
    }
}

pub fn simulate_route(_capability: &str, _model: &str) -> Result<RoutingDecision> {
    Ok(RoutingDecision::new(
        "provider-openai-official",
        vec!["provider-openai-official".into()],
    )
    .with_strategy(STRATEGY_STATIC_FALLBACK)
    .with_selection_reason(
        "used static fallback because no catalog-backed or policy-backed candidates were available",
    ))
}

pub fn create_routing_policy(input: CreateRoutingPolicyInput<'_>) -> Result<RoutingPolicy> {
    ensure!(
        !input.policy_id.trim().is_empty(),
        "policy_id must not be empty"
    );
    ensure!(
        !input.capability.trim().is_empty(),
        "capability must not be empty"
    );
    ensure!(
        !input.model_pattern.trim().is_empty(),
        "model_pattern must not be empty"
    );

    let policy = RoutingPolicy::new(input.policy_id, input.capability, input.model_pattern)
        .with_enabled(input.enabled)
        .with_priority(input.priority)
        .with_strategy(
            input
                .strategy
                .unwrap_or(RoutingStrategy::DeterministicPriority),
        )
        .with_ordered_provider_ids(input.ordered_provider_ids.to_vec())
        .with_max_cost_option(input.max_cost)
        .with_max_latency_ms_option(input.max_latency_ms)
        .with_require_healthy(input.require_healthy);

    Ok(match input.default_provider_id {
        Some(default_provider_id) if !default_provider_id.trim().is_empty() => {
            policy.with_default_provider_id(default_provider_id)
        }
        _ => policy,
    })
}

pub async fn persist_routing_policy(
    store: &dyn AdminStore,
    policy: &RoutingPolicy,
) -> Result<RoutingPolicy> {
    store.insert_routing_policy(policy).await
}

pub async fn list_routing_policies(store: &dyn AdminStore) -> Result<Vec<RoutingPolicy>> {
    store.list_routing_policies().await
}

pub fn create_routing_profile(
    input: CreateRoutingProfileInput<'_>,
) -> Result<RoutingProfileRecord> {
    ensure!(
        !input.profile_id.trim().is_empty(),
        "profile_id must not be empty"
    );
    ensure!(
        !input.tenant_id.trim().is_empty(),
        "tenant_id must not be empty"
    );
    ensure!(
        !input.project_id.trim().is_empty(),
        "project_id must not be empty"
    );
    ensure!(!input.name.trim().is_empty(), "name must not be empty");
    ensure!(!input.slug.trim().is_empty(), "slug must not be empty");

    let now = current_time_millis();
    let profile = RoutingProfileRecord::new(
        input.profile_id.trim(),
        input.tenant_id.trim(),
        input.project_id.trim(),
        input.name.trim(),
        input.slug.trim(),
    )
    .with_description_option(normalize_optional_text(input.description))
    .with_active(input.active)
    .with_strategy(
        input
            .strategy
            .unwrap_or(RoutingStrategy::DeterministicPriority),
    )
    .with_ordered_provider_ids(input.ordered_provider_ids.to_vec())
    .with_default_provider_id_option(normalize_optional_text(input.default_provider_id))
    .with_max_cost_option(input.max_cost)
    .with_max_latency_ms_option(input.max_latency_ms)
    .with_require_healthy(input.require_healthy)
    .with_preferred_region_option(normalize_optional_text(input.preferred_region))
    .with_created_at_ms(now)
    .with_updated_at_ms(now);

    Ok(profile)
}

pub async fn persist_routing_profile(
    store: &dyn AdminStore,
    profile: &RoutingProfileRecord,
) -> Result<RoutingProfileRecord> {
    store.insert_routing_profile(profile).await
}

pub async fn list_routing_profiles(store: &dyn AdminStore) -> Result<Vec<RoutingProfileRecord>> {
    store.list_routing_profiles().await
}

pub async fn list_compiled_routing_snapshots(
    store: &dyn AdminStore,
) -> Result<Vec<CompiledRoutingSnapshotRecord>> {
    store.list_compiled_routing_snapshots().await
}

pub async fn list_routing_decision_logs(store: &dyn AdminStore) -> Result<Vec<RoutingDecisionLog>> {
    store.list_routing_decision_logs().await
}

pub async fn simulate_route_with_store(
    store: &dyn AdminStore,
    capability: &str,
    model: &str,
) -> Result<RoutingDecision> {
    simulate_route_with_store_context(store, capability, model, None, None).await
}

pub async fn simulate_route_with_store_seeded(
    store: &dyn AdminStore,
    capability: &str,
    model: &str,
    selection_seed: u64,
) -> Result<RoutingDecision> {
    simulate_route_with_store_context(store, capability, model, None, Some(selection_seed)).await
}

pub async fn simulate_route_with_store_context(
    store: &dyn AdminStore,
    capability: &str,
    model: &str,
    requested_region: Option<&str>,
    selection_seed: Option<u64>,
) -> Result<RoutingDecision> {
    simulate_route_with_store_selection_context(
        store,
        capability,
        model,
        RouteSelectionContext::new(RoutingDecisionSource::AdminSimulation)
            .with_requested_region_option(requested_region)
            .with_selection_seed_option(selection_seed),
    )
    .await
}

pub async fn simulate_route_with_store_selection_context(
    store: &dyn AdminStore,
    capability: &str,
    model: &str,
    context: RouteSelectionContext<'_>,
) -> Result<RoutingDecision> {
    simulate_route_with_store_inner(
        store,
        capability,
        model,
        context.tenant_id,
        context.project_id,
        context.api_key_group_id,
        normalize_region_option(context.requested_region),
        context.selection_seed,
    )
    .await
}

pub async fn select_route_with_store(
    store: &dyn AdminStore,
    capability: &str,
    route_key: &str,
    decision_source: RoutingDecisionSource,
    tenant_id: Option<&str>,
    project_id: Option<&str>,
    selection_seed: Option<u64>,
) -> Result<RoutingDecision> {
    select_route_with_store_context(
        store,
        capability,
        route_key,
        RouteSelectionContext::new(decision_source)
            .with_tenant_id_option(tenant_id)
            .with_project_id_option(project_id)
            .with_selection_seed_option(selection_seed),
    )
    .await
}

pub async fn select_route_with_store_context(
    store: &dyn AdminStore,
    capability: &str,
    route_key: &str,
    context: RouteSelectionContext<'_>,
) -> Result<RoutingDecision> {
    let decision =
        simulate_route_with_store_selection_context(store, capability, route_key, context).await?;
    let created_at_ms = current_time_millis();
    let log = RoutingDecisionLog::new(
        generate_decision_id(created_at_ms),
        context.decision_source,
        capability,
        route_key,
        decision.selected_provider_id.clone(),
        decision
            .strategy
            .clone()
            .unwrap_or_else(|| STRATEGY_STATIC_FALLBACK.to_owned()),
        created_at_ms,
    )
    .with_tenant_id_option(context.tenant_id.map(ToOwned::to_owned))
    .with_project_id_option(context.project_id.map(ToOwned::to_owned))
    .with_api_key_group_id_option(context.api_key_group_id.map(ToOwned::to_owned))
    .with_matched_policy_id_option(decision.matched_policy_id.clone())
    .with_applied_routing_profile_id_option(decision.applied_routing_profile_id.clone())
    .with_compiled_routing_snapshot_id_option(decision.compiled_routing_snapshot_id.clone())
    .with_selection_seed_option(context.selection_seed.or(decision.selection_seed))
    .with_selection_reason_option(decision.selection_reason.clone())
    .with_fallback_reason_option(decision.fallback_reason.clone())
    .with_requested_region_option(decision.requested_region.clone())
    .with_slo_state(decision.slo_applied, decision.slo_degraded)
    .with_assessments(decision.assessments.clone());
    store.insert_routing_decision_log(&log).await?;
    Ok(decision)
}

async fn simulate_route_with_store_inner(
    store: &dyn AdminStore,
    capability: &str,
    model: &str,
    tenant_id: Option<&str>,
    project_id: Option<&str>,
    api_key_group_id: Option<&str>,
    requested_region: Option<String>,
    selection_seed: Option<u64>,
) -> Result<RoutingDecision> {
    let project_preferences = match project_id {
        Some(project_id) => store.find_project_routing_preferences(project_id).await?,
        None => None,
    };
    let applied_routing_profile =
        load_group_routing_profile(store, tenant_id, project_id, api_key_group_id).await?;
    let applied_routing_profile_id = applied_routing_profile
        .as_ref()
        .map(|profile| profile.profile_id.clone());
    let requested_region = requested_region
        .or_else(|| preferred_region_from_routing_profile(applied_routing_profile.as_ref()))
        .or_else(|| preferred_region_from_preferences(project_preferences.as_ref()));

    let mut model_candidate_ids: Vec<String> = store
        .list_models_for_external_name(model)
        .await?
        .into_iter()
        .map(|entry| entry.provider_id)
        .collect();

    model_candidate_ids.sort();
    model_candidate_ids.dedup();

    let policies = store.list_routing_policies().await?;
    let matched_policy = select_policy(&policies, capability, model);
    let effective_policy = effective_routing_policy(
        matched_policy,
        project_preferences.as_ref(),
        applied_routing_profile.as_ref(),
        capability,
        model,
    );

    let providers = if model_candidate_ids.is_empty() {
        store.list_providers().await?
    } else {
        store.list_providers_for_model(model).await?
    };
    let provider_map = providers
        .into_iter()
        .map(|provider| (provider.id.clone(), provider))
        .collect::<HashMap<_, _>>();

    let candidate_ids = if model_candidate_ids.is_empty() {
        if let Some(policy) = effective_policy.as_ref() {
            let available_provider_ids = provider_map.keys().cloned().collect::<HashSet<_>>();
            let candidate_ids = policy
                .declared_provider_ids()
                .into_iter()
                .filter(|provider_id| available_provider_ids.contains(provider_id))
                .collect::<Vec<_>>();

            if candidate_ids.is_empty() {
                let compiled_snapshot_id = persist_compiled_routing_snapshot(
                    store,
                    capability,
                    model,
                    tenant_id,
                    project_id,
                    api_key_group_id,
                    matched_policy,
                    project_preferences.as_ref(),
                    applied_routing_profile_id.as_deref(),
                    effective_policy.as_ref(),
                    requested_region.as_deref(),
                    STRATEGY_STATIC_FALLBACK,
                    &[],
                )
                .await?;
                return Ok(simulate_route(capability, model)?
                    .with_applied_routing_profile_id_option(applied_routing_profile_id.clone())
                    .with_compiled_routing_snapshot_id(compiled_snapshot_id)
                    .with_fallback_reason(
                        "no catalog-backed or policy-backed candidates were available",
                    )
                    .with_requested_region_option(requested_region.clone()));
            }

            candidate_ids
        } else {
            let compiled_snapshot_id = persist_compiled_routing_snapshot(
                store,
                capability,
                model,
                tenant_id,
                project_id,
                api_key_group_id,
                matched_policy,
                project_preferences.as_ref(),
                applied_routing_profile_id.as_deref(),
                effective_policy.as_ref(),
                requested_region.as_deref(),
                STRATEGY_STATIC_FALLBACK,
                &[],
            )
            .await?;
            return Ok(simulate_route(capability, model)?
                .with_applied_routing_profile_id_option(applied_routing_profile_id.clone())
                .with_compiled_routing_snapshot_id(compiled_snapshot_id)
                .with_fallback_reason(
                    "no catalog-backed or policy-backed candidates were available",
                )
                .with_requested_region_option(requested_region.clone()));
        }
    } else {
        match effective_policy.as_ref() {
            Some(policy) => policy.rank_candidates(&model_candidate_ids),
            None => model_candidate_ids,
        }
    };

    let compiled_routing_snapshot_id = persist_compiled_routing_snapshot(
        store,
        capability,
        model,
        tenant_id,
        project_id,
        api_key_group_id,
        matched_policy,
        project_preferences.as_ref(),
        applied_routing_profile_id.as_deref(),
        effective_policy.as_ref(),
        requested_region.as_deref(),
        resolved_compiled_snapshot_strategy(effective_policy.as_ref()),
        &candidate_ids,
    )
    .await?;

    let installations = store.list_extension_installations().await?;
    let installations_by_id = installations
        .into_iter()
        .map(|installation| (installation.installation_id.clone(), installation))
        .collect::<HashMap<_, _>>();
    let instances = store.list_extension_instances().await?;
    let instances_by_provider_id = instances
        .into_iter()
        .map(|instance| (instance.instance_id.clone(), instance))
        .collect::<HashMap<_, _>>();
    let runtime_statuses = list_extension_runtime_statuses()?;
    let latest_provider_health =
        latest_provider_health_snapshots(store.list_provider_health_snapshots().await?);

    let mut assessments = candidate_ids
        .iter()
        .enumerate()
        .map(|(policy_rank, provider_id)| {
            assess_candidate(
                provider_id,
                policy_rank,
                &provider_map,
                &instances_by_provider_id,
                &installations_by_id,
                &runtime_statuses,
                latest_provider_health.get(provider_id),
            )
        })
        .collect::<Vec<_>>();
    assessments.sort_by(compare_assessments);

    let selection = select_candidate(
        &mut assessments,
        effective_policy.as_ref(),
        requested_region.as_deref(),
        selection_seed,
    );
    let CandidateSelection {
        selected_index,
        strategy,
        selection_seed,
        selection_reason,
        fallback_reason,
        slo_applied,
        slo_degraded,
    } = selection;
    let selected = assessments
        .get(selected_index)
        .map(|assessment| assessment.provider_id.clone())
        .unwrap_or_else(|| candidate_ids[0].clone());

    let ranked_candidate_ids = assessments
        .iter()
        .map(|assessment| assessment.provider_id.clone())
        .collect::<Vec<_>>();

    let mut decision = RoutingDecision::new(selected, ranked_candidate_ids)
        .with_applied_routing_profile_id_option(applied_routing_profile_id)
        .with_compiled_routing_snapshot_id(compiled_routing_snapshot_id)
        .with_strategy(strategy)
        .with_selection_reason(selection_reason)
        .with_fallback_reason_option(fallback_reason)
        .with_requested_region_option(requested_region)
        .with_slo_state(slo_applied, slo_degraded)
        .with_assessments(assessments);
    if let Some(selection_seed) = selection_seed {
        decision = decision.with_selection_seed(selection_seed);
    }
    Ok(match matched_policy {
        Some(policy) => decision.with_matched_policy_id(policy.policy_id.clone()),
        None => decision,
    })
}

async fn load_group_routing_profile(
    store: &dyn AdminStore,
    tenant_id: Option<&str>,
    project_id: Option<&str>,
    api_key_group_id: Option<&str>,
) -> Result<Option<RoutingProfileRecord>> {
    let Some(group_id) = api_key_group_id else {
        return Ok(None);
    };

    let Some(group) = store.find_api_key_group(group_id).await? else {
        return Ok(None);
    };

    if let Some(tenant_id) = tenant_id {
        if group.tenant_id != tenant_id {
            return Ok(None);
        }
    }

    if let Some(project_id) = project_id {
        if group.project_id != project_id {
            return Ok(None);
        }
    }

    let Some(profile_id) = group.default_routing_profile_id.as_deref() else {
        return Ok(None);
    };

    let Some(profile) = store.find_routing_profile(profile_id).await? else {
        return Ok(None);
    };

    if !profile.active
        || profile.tenant_id != group.tenant_id
        || profile.project_id != group.project_id
    {
        return Ok(None);
    }

    Ok(Some(profile))
}

fn preferred_region_from_preferences(
    preferences: Option<&ProjectRoutingPreferences>,
) -> Option<String> {
    preferences.and_then(|preferences| preferences.preferred_region.clone())
}

fn preferred_region_from_routing_profile(profile: Option<&RoutingProfileRecord>) -> Option<String> {
    profile.and_then(|profile| profile.preferred_region.clone())
}

fn effective_routing_policy(
    matched_policy: Option<&RoutingPolicy>,
    preferences: Option<&ProjectRoutingPreferences>,
    routing_profile: Option<&RoutingProfileRecord>,
    capability: &str,
    model: &str,
) -> Option<RoutingPolicy> {
    let policy =
        apply_project_routing_preferences(matched_policy.cloned(), preferences, capability, model);
    apply_routing_profile(policy, routing_profile, capability, model)
}

fn resolved_compiled_snapshot_strategy(effective_policy: Option<&RoutingPolicy>) -> &'static str {
    match effective_policy {
        Some(policy) => policy.strategy.as_str(),
        None => RoutingStrategy::DeterministicPriority.as_str(),
    }
}

async fn persist_compiled_routing_snapshot(
    store: &dyn AdminStore,
    capability: &str,
    route_key: &str,
    tenant_id: Option<&str>,
    project_id: Option<&str>,
    api_key_group_id: Option<&str>,
    matched_policy: Option<&RoutingPolicy>,
    project_preferences: Option<&ProjectRoutingPreferences>,
    applied_routing_profile_id: Option<&str>,
    effective_policy: Option<&RoutingPolicy>,
    preferred_region: Option<&str>,
    strategy: &str,
    ordered_provider_ids: &[String],
) -> Result<String> {
    let now = current_time_millis();
    let snapshot_id = build_compiled_routing_snapshot_id(
        tenant_id,
        project_id,
        api_key_group_id,
        capability,
        route_key,
    );
    let snapshot = CompiledRoutingSnapshotRecord::new(&snapshot_id, capability, route_key)
        .with_tenant_id_option(tenant_id.map(ToOwned::to_owned))
        .with_project_id_option(project_id.map(ToOwned::to_owned))
        .with_api_key_group_id_option(api_key_group_id.map(ToOwned::to_owned))
        .with_matched_policy_id_option(matched_policy.map(|policy| policy.policy_id.clone()))
        .with_project_routing_preferences_project_id_option(
            project_preferences.map(|preferences| preferences.project_id.clone()),
        )
        .with_applied_routing_profile_id_option(applied_routing_profile_id.map(ToOwned::to_owned))
        .with_strategy(strategy)
        .with_ordered_provider_ids(ordered_provider_ids.to_vec())
        .with_default_provider_id_option(
            effective_policy.and_then(|policy| policy.default_provider_id.clone()),
        )
        .with_max_cost_option(effective_policy.and_then(|policy| policy.max_cost))
        .with_max_latency_ms_option(effective_policy.and_then(|policy| policy.max_latency_ms))
        .with_require_healthy(
            effective_policy
                .map(|policy| policy.require_healthy)
                .unwrap_or(false),
        )
        .with_preferred_region_option(preferred_region.map(ToOwned::to_owned))
        .with_created_at_ms(now)
        .with_updated_at_ms(now);
    store.insert_compiled_routing_snapshot(&snapshot).await?;
    Ok(snapshot_id)
}

fn build_compiled_routing_snapshot_id(
    tenant_id: Option<&str>,
    project_id: Option<&str>,
    api_key_group_id: Option<&str>,
    capability: &str,
    route_key: &str,
) -> String {
    format!(
        "routing-snapshot-{}-{}-{}-{}-{}",
        snapshot_id_segment(tenant_id, "global"),
        snapshot_id_segment(project_id, "global"),
        snapshot_id_segment(api_key_group_id, "default"),
        sanitize_snapshot_segment(capability),
        sanitize_snapshot_segment(route_key),
    )
}

fn snapshot_id_segment(value: Option<&str>, fallback: &str) -> String {
    value
        .map(sanitize_snapshot_segment)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_owned())
}

fn sanitize_snapshot_segment(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    let mut last_was_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            sanitized.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            sanitized.push('-');
            last_was_dash = true;
        }
    }
    sanitized.trim_matches('-').to_owned()
}

fn apply_project_routing_preferences(
    base_policy: Option<RoutingPolicy>,
    preferences: Option<&ProjectRoutingPreferences>,
    capability: &str,
    model: &str,
) -> Option<RoutingPolicy> {
    let Some(preferences) = preferences else {
        return base_policy;
    };

    let base_policy = base_policy
        .unwrap_or_else(|| RoutingPolicy::new("project-routing-preferences", capability, model));

    Some(
        base_policy
            .clone()
            .with_strategy(preferences.strategy)
            .with_ordered_provider_ids(if preferences.ordered_provider_ids.is_empty() {
                base_policy.ordered_provider_ids.clone()
            } else {
                preferences.ordered_provider_ids.clone()
            })
            .with_default_provider_id_option(
                preferences
                    .default_provider_id
                    .clone()
                    .or(base_policy.default_provider_id.clone()),
            )
            .with_max_cost_option(combine_optional_f64(
                base_policy.max_cost,
                preferences.max_cost,
            ))
            .with_max_latency_ms_option(combine_optional_u64(
                base_policy.max_latency_ms,
                preferences.max_latency_ms,
            ))
            .with_require_healthy(base_policy.require_healthy || preferences.require_healthy),
    )
}

fn apply_routing_profile(
    base_policy: Option<RoutingPolicy>,
    routing_profile: Option<&RoutingProfileRecord>,
    capability: &str,
    model: &str,
) -> Option<RoutingPolicy> {
    let Some(routing_profile) = routing_profile else {
        return base_policy;
    };

    let base_policy = base_policy
        .unwrap_or_else(|| RoutingPolicy::new("group-routing-profile", capability, model));

    Some(
        base_policy
            .clone()
            .with_strategy(routing_profile.strategy)
            .with_ordered_provider_ids(if routing_profile.ordered_provider_ids.is_empty() {
                base_policy.ordered_provider_ids.clone()
            } else {
                routing_profile.ordered_provider_ids.clone()
            })
            .with_default_provider_id_option(
                routing_profile
                    .default_provider_id
                    .clone()
                    .or(base_policy.default_provider_id.clone()),
            )
            .with_max_cost_option(combine_optional_f64(
                base_policy.max_cost,
                routing_profile.max_cost,
            ))
            .with_max_latency_ms_option(combine_optional_u64(
                base_policy.max_latency_ms,
                routing_profile.max_latency_ms,
            ))
            .with_require_healthy(base_policy.require_healthy || routing_profile.require_healthy),
    )
}

fn combine_optional_f64(base: Option<f64>, overlay: Option<f64>) -> Option<f64> {
    match (base, overlay) {
        (Some(base), Some(overlay)) => Some(base.min(overlay)),
        (Some(base), None) => Some(base),
        (None, Some(overlay)) => Some(overlay),
        (None, None) => None,
    }
}

fn combine_optional_u64(base: Option<u64>, overlay: Option<u64>) -> Option<u64> {
    match (base, overlay) {
        (Some(base), Some(overlay)) => Some(base.min(overlay)),
        (Some(base), None) => Some(base),
        (None, Some(overlay)) => Some(overlay),
        (None, None) => None,
    }
}

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

struct CandidateSelection {
    selected_index: usize,
    strategy: String,
    selection_seed: Option<u64>,
    selection_reason: String,
    fallback_reason: Option<String>,
    slo_applied: bool,
    slo_degraded: bool,
}

impl From<RoutingStrategyExecutionResult> for CandidateSelection {
    fn from(value: RoutingStrategyExecutionResult) -> Self {
        Self {
            selected_index: value.selected_index,
            strategy: value.strategy,
            selection_seed: value.selection_seed,
            selection_reason: value.selection_reason,
            fallback_reason: value.fallback_reason,
            slo_applied: value.slo_applied,
            slo_degraded: value.slo_degraded,
        }
    }
}

fn select_candidate(
    assessments: &mut [RoutingCandidateAssessment],
    matched_policy: Option<&RoutingPolicy>,
    requested_region: Option<&str>,
    provided_selection_seed: Option<u64>,
) -> CandidateSelection {
    let routing_strategy = matched_policy
        .map(|policy| policy.strategy)
        .unwrap_or(RoutingStrategy::DeterministicPriority);
    let registry = builtin_routing_strategy_registry();
    let plugin = registry.resolve(routing_strategy).unwrap_or_else(|| {
        registry
            .resolve(RoutingStrategy::DeterministicPriority)
            .expect("builtin deterministic-priority routing plugin must exist")
    });

    plugin
        .execute(RoutingStrategyExecutionInput {
            assessments,
            matched_policy,
            requested_region,
            provided_selection_seed,
        })
        .into()
}

fn generate_selection_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| {
            let nanos = duration.as_nanos();
            (nanos ^ u128::from(std::process::id())) as u64
        })
        .unwrap_or_else(|_| u64::from(std::process::id()))
}

fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

fn generate_decision_id(created_at_ms: u64) -> String {
    format!("route-dec-{}-{}", created_at_ms, generate_selection_seed())
}

fn assess_candidate(
    provider_id: &str,
    policy_rank: usize,
    provider_map: &HashMap<String, ProxyProvider>,
    instances_by_provider_id: &HashMap<String, ExtensionInstance>,
    installations_by_id: &HashMap<String, ExtensionInstallation>,
    runtime_statuses: &[ExtensionRuntimeStatusRecord],
    persisted_provider_health: Option<&ProviderHealthSnapshot>,
) -> RoutingCandidateAssessment {
    let mut assessment = RoutingCandidateAssessment::new(provider_id).with_policy_rank(policy_rank);

    let Some(provider) = provider_map.get(provider_id) else {
        return assessment
            .with_available(false)
            .with_health(RoutingCandidateHealth::Unknown)
            .with_reason("provider record is missing from the catalog");
    };

    let instance = instances_by_provider_id.get(provider_id);
    let mut available = true;

    if let Some(instance) = instance {
        match installations_by_id.get(&instance.installation_id) {
            Some(installation) => {
                if !installation.enabled {
                    available = false;
                    assessment = assessment.with_reason("extension installation is disabled");
                }
            }
            None => {
                assessment = assessment.with_reason(
                    "matching extension instance has no installation record, direct provider fallback may apply",
                );
            }
        }

        if !instance.enabled {
            available = false;
            assessment = assessment.with_reason("matching extension instance is disabled");
        }

        if let Some(weight) = routing_hint_u64(&instance.config, "weight") {
            assessment = assessment.with_weight(weight);
        }
        if let Some(cost) = routing_hint_f64(&instance.config, "cost") {
            assessment = assessment.with_cost(cost);
        }
        if let Some(latency_ms) = routing_hint_u64(&instance.config, "latency_ms") {
            assessment = assessment.with_latency_ms(latency_ms);
        }
        if let Some(region) = routing_hint_string(&instance.config, "region") {
            assessment = assessment.with_region(region);
        }
    } else {
        assessment =
            assessment.with_reason("no matching extension instance is mounted for this provider");
    }

    assessment = assessment.with_available(available);

    let matching_statuses =
        matching_runtime_statuses_for_provider(provider, instance, runtime_statuses);
    if matching_statuses.is_empty() {
        if let Some(snapshot) = persisted_provider_health {
            let health = if snapshot.healthy {
                RoutingCandidateHealth::Healthy
            } else {
                RoutingCandidateHealth::Unhealthy
            };
            assessment = assessment.with_health(health).with_reason(format!(
                "used persisted runtime health snapshot from {} at {}",
                snapshot.runtime, snapshot.observed_at_ms
            ));
            if let Some(message) = &snapshot.message {
                assessment = assessment.with_reason(format!("snapshot message = {message}"));
            }
        } else {
            assessment = assessment
                .with_health(RoutingCandidateHealth::Unknown)
                .with_reason("no runtime health signal is available");
        }
    } else if matching_statuses.iter().any(|status| status.healthy) {
        let runtime = matching_statuses[0].runtime.as_str();
        assessment = assessment
            .with_health(RoutingCandidateHealth::Healthy)
            .with_reason(format!("healthy runtime signal from {runtime}"));
    } else {
        let runtime = matching_statuses[0].runtime.as_str();
        assessment = assessment
            .with_health(RoutingCandidateHealth::Unhealthy)
            .with_reason(format!("runtime signal from {runtime} is unhealthy"));
    }

    if assessment.weight.is_none() {
        assessment = assessment.with_reason("default routing weight applies");
    }
    if let Some(cost) = assessment.cost {
        assessment = assessment.with_reason(format!("cost hint = {cost}"));
    }
    if let Some(latency_ms) = assessment.latency_ms {
        assessment = assessment.with_reason(format!("latency hint = {latency_ms}ms"));
    }
    if let Some(region) = assessment.region.clone() {
        assessment = assessment.with_reason(format!("region hint = {region}"));
    }

    assessment
}

fn latest_provider_health_snapshots(
    snapshots: Vec<ProviderHealthSnapshot>,
) -> HashMap<String, ProviderHealthSnapshot> {
    let mut latest = HashMap::new();
    for snapshot in snapshots {
        latest
            .entry(snapshot.provider_id.clone())
            .or_insert(snapshot);
    }
    latest
}

fn compare_assessments(
    left: &RoutingCandidateAssessment,
    right: &RoutingCandidateAssessment,
) -> Ordering {
    right
        .available
        .cmp(&left.available)
        .then_with(|| health_rank(&right.health).cmp(&health_rank(&left.health)))
        .then_with(|| left.policy_rank.cmp(&right.policy_rank))
        .then_with(|| compare_option_f64_asc(left.cost, right.cost))
        .then_with(|| compare_option_u64_asc(left.latency_ms, right.latency_ms))
        .then_with(|| {
            compare_option_u64_desc(
                Some(resolved_weight(left.weight)),
                Some(resolved_weight(right.weight)),
            )
        })
        .then_with(|| left.provider_id.cmp(&right.provider_id))
}

fn health_rank(health: &RoutingCandidateHealth) -> u8 {
    match health {
        RoutingCandidateHealth::Healthy => 2,
        RoutingCandidateHealth::Unknown => 1,
        RoutingCandidateHealth::Unhealthy => 0,
    }
}

fn resolved_weight(weight: Option<u64>) -> u64 {
    weight.unwrap_or(DEFAULT_WEIGHT)
}

fn compare_option_f64_asc(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_option_u64_asc(left: Option<u64>, right: Option<u64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_option_u64_desc(left: Option<u64>, right: Option<u64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => right.cmp(&left),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn routing_hint_u64(config: &Value, key: &str) -> Option<u64> {
    config
        .get("routing")
        .and_then(|routing| routing.get(key))
        .and_then(Value::as_u64)
        .or_else(|| config.get(key).and_then(Value::as_u64))
}

fn routing_hint_f64(config: &Value, key: &str) -> Option<f64> {
    config
        .get("routing")
        .and_then(|routing| routing.get(key))
        .and_then(Value::as_f64)
        .or_else(|| config.get(key).and_then(Value::as_f64))
}

fn routing_hint_string(config: &Value, key: &str) -> Option<String> {
    config
        .get("routing")
        .and_then(|routing| routing.get(key))
        .and_then(Value::as_str)
        .or_else(|| config.get(key).and_then(Value::as_str))
        .and_then(normalize_region)
}

fn normalize_region_option(region: Option<&str>) -> Option<String> {
    region.and_then(normalize_region)
}

fn normalize_region(region: &str) -> Option<String> {
    let normalized = region.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}
