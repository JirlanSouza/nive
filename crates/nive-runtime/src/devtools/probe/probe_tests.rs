use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestProbe {
    CreateProject,
    TagList,
}

impl ProbeCatalogEntry for TestProbe {
    const ALL: &'static [Self] = &[Self::CreateProject, Self::TagList];

    fn meta(self) -> ProbeMeta {
        match self {
            Self::CreateProject => ProbeMeta::new(
                "project_catalog.create",
                "create_project",
                "Couldn't create project",
                ProbeErrorScope::Custom("project_catalog"),
            ),
            Self::TagList => ProbeMeta::new(
                "tag.list",
                "list_tags",
                "Couldn't load tags",
                ProbeErrorScope::Custom("tag"),
            ),
        }
    }
}

#[test]
fn matches_key_or_short_key_after_normalization() {
    let meta = ProbeMeta::new(
        "project_catalog.create",
        "create_project",
        "Couldn't create project",
        ProbeErrorScope::Custom("project_catalog"),
    );

    assert!(meta.matches_name("project_catalog.create"));
    assert!(meta.matches_name("create_project"));
    assert!(meta.matches_name("create-project"));
    assert!(!meta.matches_name("delete_project"));
}

#[test]
fn default_error_includes_probe_key() {
    let meta = ProbeMeta::new(
        "project_catalog.create",
        "create_project",
        "Couldn't create project",
        ProbeErrorScope::Custom("project_catalog"),
    );

    let error = meta.error(None);

    assert_eq!(
        error.detail(),
        "Couldn't create project (probe: project_catalog.create)"
    );
    assert_eq!(error.summary(), "Couldn't create project");
}

#[test]
fn catalog_entry_exposes_default_matching_and_error_helpers() {
    assert_eq!(TestProbe::ALL[0], TestProbe::CreateProject);
    assert_eq!(TestProbe::CreateProject.key(), "project_catalog.create");
    assert!(TestProbe::CreateProject.matches_name("create-project"));
    assert_eq!(
        TestProbe::CreateProject.error(None).summary(),
        "Couldn't create project"
    );
}

#[test]
fn parses_probe_config_for_generic_catalog() {
    let config = parse_probe_config::<TestProbe>("create-project, tag.list:delay=2s;message=Boom");

    assert_eq!(config.unknown, Vec::<String>::new());
    assert_eq!(config.scenarios.len(), 2);
    assert_eq!(config.scenarios[0].probe, TestProbe::CreateProject);
    assert_eq!(config.scenarios[1].probe, TestProbe::TagList);
    assert_eq!(config.scenarios[1].delay, Some(Duration::from_secs(2)));
    assert_eq!(config.scenarios[1].message.as_deref(), Some("Boom"));
}

#[test]
fn expands_all_and_collects_unknown_probe_names() {
    let config = parse_probe_config::<TestProbe>("all, nope");

    let probes: Vec<_> = config
        .scenarios
        .iter()
        .map(|scenario| scenario.probe)
        .collect();

    assert_eq!(probes, TestProbe::ALL.to_vec());
    assert_eq!(config.unknown, vec!["nope".to_string()]);
}

#[test]
fn scenario_fire_state_handles_skip_count_and_repeat() {
    let mut config = parse_probe_config::<TestProbe>("tag.list:skip=2;count=1;repeat=true");
    let scenario = &mut config.scenarios[0];

    assert!(!scenario.should_fire());
    assert!(!scenario.should_fire());
    assert!(scenario.should_fire());
    assert_eq!(scenario.skip_remaining, Some(2));
    assert_eq!(scenario.remaining, Some(1));

    assert!(!scenario.should_fire());
    assert!(!scenario.should_fire());
    assert!(scenario.should_fire());
}

#[test]
fn runtime_config_builds_enabled_scenario_or_none() {
    assert!(ProbeRuntimeConfig::disabled()
        .into_scenario(TestProbe::TagList)
        .is_none());

    let scenario = ProbeRuntimeConfig {
        enabled: true,
        effect: ProbeEffect::DelayOnly,
        delay: Some(Duration::from_millis(250)),
        skip: Some(1),
        count: Some(2),
        repeat: true,
        message: Some("Wait".to_string()),
    }
    .into_scenario(TestProbe::TagList)
    .expect("enabled probe should create a scenario");

    assert_eq!(scenario.probe, TestProbe::TagList);
    assert_eq!(scenario.effect, ProbeEffect::DelayOnly);
    assert_eq!(scenario.delay, Some(Duration::from_millis(250)));
    assert_eq!(scenario.skip_remaining, Some(1));
    assert_eq!(scenario.remaining, Some(2));
}

#[test]
fn catalog_helpers_return_keys_and_display_items() {
    let items = probe_catalog_items::<TestProbe>();

    assert_eq!(
        items,
        vec![
            ProbeCatalogItem {
                key: "project_catalog.create",
                short_key: "create_project",
                summary: "Couldn't create project",
            },
            ProbeCatalogItem {
                key: "tag.list",
                short_key: "list_tags",
                summary: "Couldn't load tags",
            },
        ]
    );
    assert_eq!(
        probe_catalog_keys::<TestProbe>(),
        vec!["project_catalog.create", "tag.list"]
    );
}

#[test]
fn probe_meta_catalog_flattens_groups_and_finds_names() {
    const PROJECT: &[ProbeMeta] = &[ProbeMeta::new(
        "project_catalog.create",
        "create_project",
        "Couldn't create project",
        ProbeErrorScope::Custom("project_catalog"),
    )];
    const TAGS: &[ProbeMeta] = &[ProbeMeta::new(
        "tag.list",
        "list_tags",
        "Couldn't load tags",
        ProbeErrorScope::Custom("tag"),
    )];
    const CATALOG: ProbeMetaCatalog = ProbeMetaCatalog::new(&[PROJECT, TAGS]);

    assert_eq!(CATALOG.len(), 2);
    assert!(!CATALOG.is_empty());
    assert_eq!(CATALOG.get(0), Some(PROJECT[0]));
    assert_eq!(CATALOG.get(1), Some(TAGS[0]));
    assert_eq!(CATALOG.get(2), None);
    assert_eq!(CATALOG.to_vec(), vec![PROJECT[0], TAGS[0]]);
    assert_eq!(CATALOG.index_by_name("create-project"), Some(0));
    assert_eq!(CATALOG.index_by_name("tag.list"), Some(1));
    assert_eq!(CATALOG.index_by_name("missing"), None);
}

#[test]
fn composed_probe_ids_join_app_and_generated_probe_metadata() {
    const APP: &[ProbeMeta] = &[ProbeMeta::new(
        "bootstrap",
        "bootstrap",
        "Couldn't initialize the app",
        ProbeErrorScope::Bootstrap,
    )];
    const CLIENTS: &[ProbeMeta] = &[ProbeMeta::new(
        "tag.list",
        "list_tags",
        "Couldn't load tags",
        ProbeErrorScope::Custom("tag"),
    )];
    const CATALOG: ProbeMetaCatalog = ProbeMetaCatalog::new(&[CLIENTS]);
    const IDS: [ComposedProbeId; 2] = composed_probe_ids::<2>(APP.len(), CATALOG.len());

    assert_eq!(
        IDS,
        [ComposedProbeId::App(0), ComposedProbeId::Generated(0)]
    );
    assert_eq!(IDS[0].meta(APP, CATALOG), Some(APP[0]));
    assert_eq!(IDS[1].meta(APP, CATALOG), Some(CLIENTS[0]));
    assert_eq!(
        ComposedProbeId::generated_by_name("list-tags", CATALOG),
        Some(ComposedProbeId::Generated(0))
    );
}

#[test]
fn snapshot_reports_active_probe_keys() {
    let store = ProbeInjectionStore::<TestProbe>::from_raw("create_project, tag.list");

    assert_eq!(
        store.snapshot().active_probe_keys(),
        vec!["project_catalog.create", "tag.list"]
    );
}

#[test]
fn drafts_include_all_catalog_entries_with_snapshot_state() {
    let store =
        ProbeInjectionStore::<TestProbe>::from_raw("tag.list:delay=250ms;skip=2;message=Boom");

    let drafts = probe_drafts_from_snapshot(&store.snapshot());

    assert_eq!(drafts.len(), 2);
    assert_eq!(drafts[0], ProbeDraft::disabled(TestProbe::CreateProject));
    assert_eq!(drafts[1].probe, TestProbe::TagList);
    assert!(drafts[1].enabled);
    assert_eq!(drafts[1].delay_ms, "250");
    assert_eq!(drafts[1].skip, "2");
    assert_eq!(drafts[1].message, "Boom");
}

#[test]
fn draft_runtime_config_parses_numeric_inputs() {
    let mut draft = ProbeDraft::disabled(TestProbe::TagList);

    draft.set_enabled(true);
    draft.set_effect(ProbeEffect::DelayOnly);
    draft.set_delay_input("250ms".to_string());
    draft.set_skip_input(" 2 ".to_string());
    draft.set_count_input("1x".to_string());
    draft.set_repeat(true);
    draft.set_message("Wait".to_string());

    let config = draft.runtime_config();

    assert!(config.enabled);
    assert_eq!(config.effect, ProbeEffect::DelayOnly);
    assert_eq!(config.delay, Some(Duration::from_millis(250)));
    assert_eq!(config.skip, Some(2));
    assert_eq!(config.count, Some(1));
    assert!(config.repeat);
    assert_eq!(config.message.as_deref(), Some("Wait"));
}

#[test]
fn draft_fire_next_enables_single_failure_without_repeat() {
    let mut draft = ProbeDraft::disabled(TestProbe::TagList);
    draft.skip = "2".to_string();
    draft.repeat = true;

    draft.fire_next();

    assert!(draft.enabled);
    assert_eq!(draft.skip, "");
    assert_eq!(draft.count, "1");
    assert!(!draft.repeat);
}

#[test]
fn draft_filter_matches_active_state_key_and_short_key() {
    let disabled = ProbeDraft::disabled(TestProbe::CreateProject);
    let enabled = ProbeDraft {
        enabled: true,
        ..ProbeDraft::disabled(TestProbe::TagList)
    };

    assert!(disabled.matches_filter(false, "create_project"));
    assert!(enabled.matches_filter(true, "list_tags"));
    assert!(!disabled.matches_filter(true, "create-project"));
    assert!(!enabled.matches_filter(false, "missing"));
}

#[test]
fn draft_summary_describes_enabled_runtime_config() {
    let disabled = ProbeDraft::disabled(TestProbe::CreateProject);
    let enabled = ProbeDraft {
        enabled: true,
        effect: ProbeEffect::Fail,
        delay_ms: "250".to_string(),
        skip: "2".to_string(),
        count: "1".to_string(),
        repeat: true,
        message: "Boom".to_string(),
        ..ProbeDraft::disabled(TestProbe::TagList)
    };
    let delay_only = ProbeDraft {
        enabled: true,
        effect: ProbeEffect::DelayOnly,
        ..ProbeDraft::disabled(TestProbe::TagList)
    };

    assert_eq!(disabled.summary(), "disabled");
    assert_eq!(
        enabled.summary(),
        "fail · delay 250ms · skip 2 · count 1 · repeat · custom message"
    );
    assert_eq!(delay_only.summary(), "delay-only");
}

#[test]
fn update_probe_drafts_mutates_draft_and_returns_store_effect() {
    let mut drafts = vec![ProbeDraft::disabled(TestProbe::TagList)];

    let effect = update_probe_drafts(
        &mut drafts,
        ProbePanelMessage::DelayChanged(TestProbe::TagList, "250ms".to_string()),
    );

    assert_eq!(drafts[0].delay_ms, "250");
    assert_eq!(
        effect,
        Some(ProbePanelEffect::SetProbeConfig(
            TestProbe::TagList,
            ProbeRuntimeConfig {
                enabled: false,
                effect: ProbeEffect::Fail,
                delay: Some(Duration::from_millis(250)),
                skip: None,
                count: None,
                repeat: false,
                message: None,
            }
        ))
    );
}

#[test]
fn update_probe_drafts_clears_probe_or_all_drafts() {
    let mut drafts = vec![
        ProbeDraft {
            enabled: true,
            ..ProbeDraft::disabled(TestProbe::CreateProject)
        },
        ProbeDraft {
            enabled: true,
            ..ProbeDraft::disabled(TestProbe::TagList)
        },
    ];

    let effect = update_probe_drafts(
        &mut drafts,
        ProbePanelMessage::ClearProbe(TestProbe::TagList),
    );

    assert!(drafts[0].enabled);
    assert!(!drafts[1].enabled);
    assert_eq!(
        effect,
        Some(ProbePanelEffect::SetProbeConfig(
            TestProbe::TagList,
            ProbeRuntimeConfig::disabled()
        ))
    );

    let effect = update_probe_drafts(&mut drafts, ProbePanelMessage::ClearAll);

    assert!(!drafts[0].enabled);
    assert!(!drafts[1].enabled);
    assert_eq!(effect, Some(ProbePanelEffect::ClearAll));
}

#[test]
fn store_replaces_existing_probe_config() {
    let mut store = ProbeInjectionStore::<TestProbe>::from_raw("tag.list:count=3");

    store.set_probe_config(
        TestProbe::TagList,
        ProbeRuntimeConfig {
            enabled: true,
            effect: ProbeEffect::Fail,
            delay: Some(Duration::from_millis(250)),
            skip: Some(1),
            count: Some(2),
            repeat: true,
            message: Some("Boom".to_string()),
        },
    );

    let snapshot = store.snapshot();

    assert_eq!(snapshot.scenarios.len(), 1);
    assert_eq!(snapshot.scenarios[0].probe, TestProbe::TagList);
    assert_eq!(
        snapshot.scenarios[0].delay,
        Some(Duration::from_millis(250))
    );
    assert_eq!(snapshot.scenarios[0].skip, Some(1));
    assert_eq!(snapshot.scenarios[0].count, Some(2));
    assert!(snapshot.scenarios[0].repeat);
    assert_eq!(snapshot.scenarios[0].message.as_deref(), Some("Boom"));
}

#[test]
fn delay_only_store_injection_has_no_error() {
    let mut store = ProbeInjectionStore::<TestProbe>::from_raw("tag.list:effect=delay;delay=250ms");

    let injection = store
        .inject(TestProbe::TagList)
        .expect("delay-only probe should inject");

    assert_eq!(injection.effect(), ProbeEffect::DelayOnly);
    assert_eq!(injection.delay(), Some(Duration::from_millis(250)));
    assert!(injection.error().is_none());
}

#[test]
fn store_injects_by_key_or_short_key() {
    let mut store = ProbeInjectionStore::<TestProbe>::from_raw("tag.list, create_project");

    assert!(store.inject_by_name("tag.list").is_some());
    assert!(store.inject_by_name("create_project").is_some());
    assert!(store.inject_by_name("missing").is_none());
}

#[test]
fn disabled_probe_config_removes_existing_store_config() {
    let mut store = ProbeInjectionStore::<TestProbe>::from_raw("tag.list:count=3");

    store.set_probe_config(TestProbe::TagList, ProbeRuntimeConfig::disabled());

    assert!(store.snapshot().scenarios.is_empty());
}
