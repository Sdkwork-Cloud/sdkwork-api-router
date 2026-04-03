use sdkwork_api_domain_routing::{
    RoutingCandidateAssessment, RoutingCandidateHealth, RoutingPolicy, RoutingStrategy,
};
use sdkwork_api_policy_routing::{
    builtin_routing_strategy_registry, RoutingStrategyExecutionInput,
};

#[test]
fn builtin_registry_resolves_every_current_routing_strategy() {
    let registry = builtin_routing_strategy_registry();

    for strategy in [
        RoutingStrategy::DeterministicPriority,
        RoutingStrategy::WeightedRandom,
        RoutingStrategy::SloAware,
        RoutingStrategy::GeoAffinity,
    ] {
        assert!(
            registry.resolve(strategy).is_some(),
            "missing builtin plugin for strategy {}",
            strategy.as_str()
        );
    }
}

#[test]
fn weighted_random_plugin_keeps_seeded_selection_deterministic() {
    let registry = builtin_routing_strategy_registry();
    let plugin = registry
        .resolve(RoutingStrategy::WeightedRandom)
        .expect("missing weighted-random plugin");
    let policy = RoutingPolicy::new("policy-weighted", "chat_completion", "gpt-4.1")
        .with_strategy(RoutingStrategy::WeightedRandom)
        .with_ordered_provider_ids(vec![
            "provider-light".to_owned(),
            "provider-heavy".to_owned(),
        ]);
    let mut assessments = vec![
        RoutingCandidateAssessment::new("provider-light")
            .with_health(RoutingCandidateHealth::Healthy)
            .with_weight(10)
            .with_policy_rank(0),
        RoutingCandidateAssessment::new("provider-heavy")
            .with_health(RoutingCandidateHealth::Healthy)
            .with_weight(90)
            .with_policy_rank(1),
    ];

    let result = plugin.execute(RoutingStrategyExecutionInput {
        assessments: &mut assessments,
        matched_policy: Some(&policy),
        requested_region: None,
        provided_selection_seed: Some(15),
    });

    assert_eq!(result.selected_index, 1);
    assert_eq!(result.selection_seed, Some(15));
    assert_eq!(result.strategy, "weighted_random");
}

#[test]
fn geo_affinity_plugin_reports_fallback_when_requested_region_is_missing() {
    let registry = builtin_routing_strategy_registry();
    let plugin = registry
        .resolve(RoutingStrategy::GeoAffinity)
        .expect("missing geo-affinity plugin");
    let policy = RoutingPolicy::new("policy-geo", "chat_completion", "gpt-4.1")
        .with_strategy(RoutingStrategy::GeoAffinity)
        .with_ordered_provider_ids(vec![
            "provider-eu".to_owned(),
            "provider-us".to_owned(),
        ]);
    let mut assessments = vec![
        RoutingCandidateAssessment::new("provider-eu")
            .with_health(RoutingCandidateHealth::Healthy)
            .with_region("eu-west")
            .with_policy_rank(0),
        RoutingCandidateAssessment::new("provider-us")
            .with_health(RoutingCandidateHealth::Healthy)
            .with_region("us-east")
            .with_policy_rank(1),
    ];

    let result = plugin.execute(RoutingStrategyExecutionInput {
        assessments: &mut assessments,
        matched_policy: Some(&policy),
        requested_region: None,
        provided_selection_seed: None,
    });

    assert_eq!(result.selected_index, 0);
    assert_eq!(result.strategy, "geo_affinity");
    assert_eq!(
        result.fallback_reason.as_deref(),
        Some("no requested region was provided")
    );
}
