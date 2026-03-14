use std::cmp::Ordering;
use std::collections::HashMap;

use anyhow::{ensure, Result};
use sdkwork_api_app_extension::{list_extension_runtime_statuses, ExtensionRuntimeStatusRecord};
use sdkwork_api_domain_catalog::ProxyProvider;
use sdkwork_api_domain_routing::{
    select_policy, RoutingCandidateAssessment, RoutingCandidateHealth, RoutingDecision,
    RoutingPolicy,
};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance};
use sdkwork_api_storage_core::AdminStore;
use serde_json::Value;

const DEFAULT_WEIGHT: u64 = 100;
const STRATEGY_RUNTIME_AWARE: &str = "runtime_aware_deterministic";
const STRATEGY_STATIC_FALLBACK: &str = "static_fallback";

pub fn service_name() -> &'static str {
    "routing-service"
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

pub fn create_routing_policy(
    policy_id: &str,
    capability: &str,
    model_pattern: &str,
    enabled: bool,
    priority: i32,
    ordered_provider_ids: &[String],
    default_provider_id: Option<&str>,
) -> Result<RoutingPolicy> {
    ensure!(!policy_id.trim().is_empty(), "policy_id must not be empty");
    ensure!(
        !capability.trim().is_empty(),
        "capability must not be empty"
    );
    ensure!(
        !model_pattern.trim().is_empty(),
        "model_pattern must not be empty"
    );

    let policy = RoutingPolicy::new(policy_id, capability, model_pattern)
        .with_enabled(enabled)
        .with_priority(priority)
        .with_ordered_provider_ids(ordered_provider_ids.to_vec());

    Ok(match default_provider_id {
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

pub async fn simulate_route_with_store(
    store: &dyn AdminStore,
    capability: &str,
    model: &str,
) -> Result<RoutingDecision> {
    let mut model_candidate_ids: Vec<String> = store
        .list_models()
        .await?
        .into_iter()
        .filter(|entry| entry.external_name == model)
        .map(|entry| entry.provider_id)
        .collect();

    model_candidate_ids.sort();
    model_candidate_ids.dedup();

    let policies = store.list_routing_policies().await?;
    let matched_policy = select_policy(&policies, capability, model);

    let providers = store.list_providers().await?;
    let provider_map = providers
        .into_iter()
        .map(|provider| (provider.id.clone(), provider))
        .collect::<HashMap<_, _>>();

    let candidate_ids = if model_candidate_ids.is_empty() {
        if let Some(policy) = matched_policy {
            let available_provider_ids = provider_map.keys().cloned().collect::<Vec<_>>();
            let candidate_ids = policy
                .declared_provider_ids()
                .into_iter()
                .filter(|provider_id| available_provider_ids.iter().any(|id| id == provider_id))
                .collect::<Vec<_>>();

            if candidate_ids.is_empty() {
                return simulate_route(capability, model);
            }

            candidate_ids
        } else {
            return simulate_route(capability, model);
        }
    } else {
        match matched_policy {
            Some(policy) => policy.rank_candidates(&model_candidate_ids),
            None => model_candidate_ids,
        }
    };

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
            )
        })
        .collect::<Vec<_>>();
    assessments.sort_by(compare_assessments);

    let selected = assessments
        .first()
        .map(|assessment| assessment.provider_id.clone())
        .unwrap_or_else(|| candidate_ids[0].clone());
    let selected_assessment = assessments
        .first()
        .expect("routing assessments should not be empty");
    let selection_reason = if selected_assessment.reasons.is_empty() {
        format!(
            "selected {} as the top-ranked candidate",
            selected_assessment.provider_id
        )
    } else {
        format!(
            "selected {} because {}",
            selected_assessment.provider_id,
            selected_assessment.reasons.join(", ")
        )
    };

    let ranked_candidate_ids = assessments
        .iter()
        .map(|assessment| assessment.provider_id.clone())
        .collect::<Vec<_>>();

    let decision = RoutingDecision::new(selected, ranked_candidate_ids)
        .with_strategy(STRATEGY_RUNTIME_AWARE)
        .with_selection_reason(selection_reason)
        .with_assessments(assessments);
    Ok(match matched_policy {
        Some(policy) => decision.with_matched_policy_id(policy.policy_id.clone()),
        None => decision,
    })
}

fn assess_candidate(
    provider_id: &str,
    policy_rank: usize,
    provider_map: &HashMap<String, ProxyProvider>,
    instances_by_provider_id: &HashMap<String, ExtensionInstance>,
    installations_by_id: &HashMap<String, ExtensionInstallation>,
    runtime_statuses: &[ExtensionRuntimeStatusRecord],
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
    } else {
        assessment =
            assessment.with_reason("no matching extension instance is mounted for this provider");
    }

    assessment = assessment.with_available(available);

    let matching_statuses = matching_runtime_statuses(provider, instance, runtime_statuses);
    if matching_statuses.is_empty() {
        assessment = assessment
            .with_health(RoutingCandidateHealth::Unknown)
            .with_reason("no runtime health signal is available");
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

    assessment
}

fn matching_runtime_statuses<'a>(
    provider: &ProxyProvider,
    instance: Option<&ExtensionInstance>,
    runtime_statuses: &'a [ExtensionRuntimeStatusRecord],
) -> Vec<&'a ExtensionRuntimeStatusRecord> {
    if let Some(instance) = instance {
        let exact = runtime_statuses
            .iter()
            .filter(|status| status.instance_id == instance.instance_id)
            .collect::<Vec<_>>();
        if !exact.is_empty() {
            return exact;
        }
    }

    runtime_statuses
        .iter()
        .filter(|status| {
            status.extension_id == provider.extension_id && status.instance_id.is_empty()
        })
        .collect()
}

fn compare_assessments(
    left: &RoutingCandidateAssessment,
    right: &RoutingCandidateAssessment,
) -> Ordering {
    right
        .available
        .cmp(&left.available)
        .then_with(|| health_rank(&right.health).cmp(&health_rank(&left.health)))
        .then_with(|| compare_option_f64_asc(left.cost, right.cost))
        .then_with(|| compare_option_u64_asc(left.latency_ms, right.latency_ms))
        .then_with(|| {
            compare_option_u64_desc(
                Some(resolved_weight(left.weight)),
                Some(resolved_weight(right.weight)),
            )
        })
        .then_with(|| left.policy_rank.cmp(&right.policy_rank))
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
