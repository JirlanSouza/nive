use std::future::Future;
use std::time::Duration;

use iced::Task;

use crate::{UserFacingError, UserFacingResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeEffect {
    Fail,
    DelayOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientTaskInjection {
    kind: ClientTaskInjectionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ClientTaskInjectionKind {
    Fail {
        error: UserFacingError,
        delay: Option<Duration>,
    },
    DelayOnly {
        delay: Option<Duration>,
    },
}

impl ClientTaskInjection {
    pub fn fail(error: UserFacingError, delay: Option<Duration>) -> Self {
        Self {
            kind: ClientTaskInjectionKind::Fail { error, delay },
        }
    }

    pub fn delay_only(delay: Option<Duration>) -> Self {
        Self {
            kind: ClientTaskInjectionKind::DelayOnly { delay },
        }
    }

    pub fn effect(&self) -> ProbeEffect {
        match self.kind {
            ClientTaskInjectionKind::Fail { .. } => ProbeEffect::Fail,
            ClientTaskInjectionKind::DelayOnly { .. } => ProbeEffect::DelayOnly,
        }
    }

    pub fn delay(&self) -> Option<Duration> {
        match self.kind {
            ClientTaskInjectionKind::Fail { delay, .. }
            | ClientTaskInjectionKind::DelayOnly { delay } => delay,
        }
    }

    pub fn error(&self) -> Option<&UserFacingError> {
        match &self.kind {
            ClientTaskInjectionKind::Fail { error, .. } => Some(error),
            ClientTaskInjectionKind::DelayOnly { .. } => None,
        }
    }
}

pub fn injected_client_task<T, Message, Fut>(
    injection: ClientTaskInjection,
    future: Fut,
    map: impl Fn(UserFacingResult<T>) -> Message + Send + 'static,
) -> Task<Message>
where
    T: Send + 'static,
    Message: Send + 'static,
    Fut: Future<Output = UserFacingResult<T>> + Send + 'static,
{
    crate::client_task(
        async move {
            match injection.kind {
                ClientTaskInjectionKind::Fail { error, delay } => {
                    if let Some(delay) = delay {
                        tokio::time::sleep(delay).await;
                    }
                    Err(error)
                }
                ClientTaskInjectionKind::DelayOnly { delay } => {
                    if let Some(delay) = delay {
                        tokio::time::sleep(delay).await;
                    }
                    future.await
                }
            }
        },
        map,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeErrorScope {
    Bootstrap,
    Custom(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeMeta {
    pub key: &'static str,
    pub short_key: &'static str,
    pub summary: &'static str,
    pub error_scope: ProbeErrorScope,
}

pub trait ProbeCatalogEntry: Copy + Eq + Send + 'static {
    const ALL: &'static [Self];

    fn meta(self) -> ProbeMeta;

    fn key(self) -> &'static str {
        self.meta().key
    }

    fn matches_name(self, normalized_name: &str) -> bool {
        self.meta().matches_name(normalized_name)
    }

    fn error(self, message_override: Option<&str>) -> UserFacingError {
        self.meta().error(message_override)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoProbe {}

impl ProbeCatalogEntry for NoProbe {
    const ALL: &'static [Self] = &[];

    fn meta(self) -> ProbeMeta {
        match self {}
    }
}

pub struct ProbeInjectionConfig<P> {
    pub scenarios: Vec<ProbeScenarioConfig<P>>,
    pub unknown: Vec<String>,
}

pub struct ProbeInjectionStore<P> {
    config: ProbeInjectionConfig<P>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeCatalogItem {
    pub key: &'static str,
    pub short_key: &'static str,
    pub summary: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeMetaCatalog {
    groups: &'static [&'static [ProbeMeta]],
}

#[derive(Debug, Clone)]
pub struct ProbeMetaCatalogIter {
    catalog: ProbeMetaCatalog,
    index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposedProbeId {
    App(usize),
    Generated(usize),
}

#[derive(Debug)]
pub struct ProbeScenarioConfig<P> {
    pub probe: P,
    pub effect: ProbeEffect,
    pub delay: Option<Duration>,
    pub skip: Option<u32>,
    pub skip_remaining: Option<u32>,
    pub count: Option<u32>,
    pub remaining: Option<u32>,
    pub repeat: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeRuntimeConfig {
    pub enabled: bool,
    pub effect: ProbeEffect,
    pub delay: Option<Duration>,
    pub skip: Option<u32>,
    pub count: Option<u32>,
    pub repeat: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeInjectionSnapshot<P> {
    pub scenarios: Vec<ProbeScenarioSnapshot<P>>,
    pub unknown: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeScenarioSnapshot<P> {
    pub probe: P,
    pub effect: ProbeEffect,
    pub delay: Option<Duration>,
    pub skip: Option<u32>,
    pub skip_remaining: Option<u32>,
    pub count: Option<u32>,
    pub remaining: Option<u32>,
    pub repeat: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeDraft<P> {
    pub probe: P,
    pub enabled: bool,
    pub effect: ProbeEffect,
    pub delay_ms: String,
    pub skip: String,
    pub count: String,
    pub repeat: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbePanelMessage<P> {
    SetProbeEnabled(P, bool),
    EffectChanged(P, ProbeEffect),
    DelayChanged(P, String),
    SkipChanged(P, String),
    CountChanged(P, String),
    RepeatChanged(P, bool),
    MessageChanged(P, String),
    FireNext(P),
    ClearProbe(P),
    ClearAll,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbePanelEffect<P> {
    SetProbeConfig(P, ProbeRuntimeConfig),
    ClearAll,
}

impl ProbeMeta {
    pub const fn new(
        key: &'static str,
        short_key: &'static str,
        summary: &'static str,
        error_scope: ProbeErrorScope,
    ) -> Self {
        Self {
            key,
            short_key,
            summary,
            error_scope,
        }
    }

    pub fn matches_name(self, normalized_name: &str) -> bool {
        let name = normalize_probe_name(normalized_name);
        name == normalize_probe_name(self.key) || name == normalize_probe_name(self.short_key)
    }

    pub fn error(self, message_override: Option<&str>) -> UserFacingError {
        let message = match message_override {
            Some(custom) => custom.to_string(),
            None => format!("{} (probe: {})", self.summary, self.key),
        };

        match self.error_scope {
            ProbeErrorScope::Bootstrap => UserFacingError::bootstrap(message),
            ProbeErrorScope::Custom(kind) => UserFacingError::custom(kind, message),
        }
    }
}

impl ProbeMetaCatalog {
    pub const fn new(groups: &'static [&'static [ProbeMeta]]) -> Self {
        Self { groups }
    }

    pub const fn len(self) -> usize {
        let mut total = 0usize;
        let mut index = 0usize;

        while index < self.groups.len() {
            total += self.groups[index].len();
            index += 1;
        }

        total
    }

    pub const fn is_empty(self) -> bool {
        self.len() == 0
    }

    pub fn get(self, mut index: usize) -> Option<ProbeMeta> {
        for group in self.groups {
            if index < group.len() {
                return Some(group[index]);
            }

            index -= group.len();
        }

        None
    }

    pub fn iter(self) -> ProbeMetaCatalogIter {
        ProbeMetaCatalogIter {
            catalog: self,
            index: 0,
        }
    }

    pub fn to_vec(self) -> Vec<ProbeMeta> {
        self.iter().collect()
    }

    pub fn index_by_name(self, name: &str) -> Option<usize> {
        let normalized_name = normalize_probe_name(name);

        self.iter()
            .position(|meta| meta.matches_name(&normalized_name))
    }
}

impl ComposedProbeId {
    pub const fn app(index: usize) -> Self {
        Self::App(index)
    }

    pub const fn generated(index: usize) -> Self {
        Self::Generated(index)
    }

    pub fn meta(
        self,
        app_probe_meta: &'static [ProbeMeta],
        generated_probe_meta: ProbeMetaCatalog,
    ) -> Option<ProbeMeta> {
        match self {
            Self::App(index) => app_probe_meta.get(index).copied(),
            Self::Generated(index) => generated_probe_meta.get(index),
        }
    }

    pub fn generated_by_name(name: &str, generated_probe_meta: ProbeMetaCatalog) -> Option<Self> {
        generated_probe_meta
            .index_by_name(name)
            .map(Self::Generated)
    }
}

pub const fn composed_probe_ids<const TOTAL: usize>(
    app_probe_count: usize,
    generated_probe_count: usize,
) -> [ComposedProbeId; TOTAL] {
    let mut probes = [ComposedProbeId::App(0); TOTAL];
    let mut index = 0usize;

    while index < app_probe_count {
        probes[index] = ComposedProbeId::App(index);
        index += 1;
    }

    let mut generated_index = 0usize;
    while generated_index < generated_probe_count {
        probes[index] = ComposedProbeId::Generated(generated_index);
        index += 1;
        generated_index += 1;
    }

    probes
}

impl Iterator for ProbeMetaCatalogIter {
    type Item = ProbeMeta;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.catalog.get(self.index)?;
        self.index += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.catalog.len().saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for ProbeMetaCatalogIter {}

impl<P: Copy> ProbeDraft<P> {
    pub fn disabled(probe: P) -> Self {
        Self {
            probe,
            enabled: false,
            effect: ProbeEffect::Fail,
            delay_ms: String::new(),
            skip: String::new(),
            count: String::new(),
            repeat: false,
            message: String::new(),
        }
    }

    pub fn from_snapshot(probe: P, snapshot: Option<&ProbeScenarioSnapshot<P>>) -> Self {
        let Some(snapshot) = snapshot else {
            return Self::disabled(probe);
        };

        Self {
            probe,
            enabled: true,
            effect: snapshot.effect,
            delay_ms: snapshot
                .delay
                .map(duration_millis_string)
                .unwrap_or_default(),
            skip: snapshot
                .skip
                .map(|value| value.to_string())
                .unwrap_or_default(),
            count: snapshot
                .count
                .map(|value| value.to_string())
                .unwrap_or_default(),
            repeat: snapshot.repeat,
            message: snapshot.message.clone().unwrap_or_default(),
        }
    }

    pub fn runtime_config(&self) -> ProbeRuntimeConfig {
        ProbeRuntimeConfig {
            enabled: self.enabled,
            effect: self.effect,
            delay: parse_duration_ms(&self.delay_ms),
            skip: parse_optional_u32(&self.skip),
            count: parse_optional_u32(&self.count),
            repeat: self.repeat,
            message: (!self.message.trim().is_empty()).then(|| self.message.clone()),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_effect(&mut self, effect: ProbeEffect) {
        self.effect = effect;
    }

    pub fn set_delay_input(&mut self, value: String) {
        self.delay_ms = numeric_input(value);
    }

    pub fn set_skip_input(&mut self, value: String) {
        self.skip = numeric_input(value);
    }

    pub fn set_count_input(&mut self, value: String) {
        self.count = numeric_input(value);
    }

    pub fn set_repeat(&mut self, repeat: bool) {
        self.repeat = repeat;
    }

    pub fn set_message(&mut self, message: String) {
        self.message = message;
    }

    pub fn fire_next(&mut self) {
        self.enabled = true;
        self.skip.clear();
        self.count = "1".to_string();
        self.repeat = false;
    }
}

impl<P: ProbeCatalogEntry> ProbeDraft<P> {
    pub fn matches_filter(&self, active_only: bool, query: &str) -> bool {
        let meta = self.probe.meta();
        (!active_only || self.enabled) && text_matches_query(query, [meta.key, meta.short_key])
    }

    pub fn summary(&self) -> String {
        if !self.enabled {
            return "disabled".to_string();
        }

        let mut parts = Vec::new();
        parts.push(match self.effect {
            ProbeEffect::Fail => "fail".to_string(),
            ProbeEffect::DelayOnly => "delay-only".to_string(),
        });

        if !self.delay_ms.is_empty() {
            parts.push(format!("delay {}ms", self.delay_ms));
        }

        if !self.skip.is_empty() {
            parts.push(format!("skip {}", self.skip));
        }

        if !self.count.is_empty() {
            parts.push(format!("count {}", self.count));
        }

        if self.repeat {
            parts.push("repeat".to_string());
        }

        if self.effect == ProbeEffect::Fail && !self.message.trim().is_empty() {
            parts.push("custom message".to_string());
        }

        parts.join(" · ")
    }
}

impl<P> ProbeScenarioConfig<P> {
    pub fn should_fire(&mut self) -> bool {
        if decrement_if_positive(&mut self.skip_remaining) {
            return false;
        }

        if self.remaining.is_none() {
            return true;
        }

        let fired = decrement_if_positive(&mut self.remaining);

        if fired && self.repeat && self.remaining == Some(0) {
            self.reset_cycle();
        }

        fired
    }

    pub fn reset_cycle(&mut self) {
        self.skip_remaining = self.skip;
        self.remaining = self.count;
    }
}

impl ProbeRuntimeConfig {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            effect: ProbeEffect::Fail,
            delay: None,
            skip: None,
            count: None,
            repeat: false,
            message: None,
        }
    }

    pub fn into_scenario<P>(self, probe: P) -> Option<ProbeScenarioConfig<P>> {
        self.enabled.then_some(ProbeScenarioConfig {
            probe,
            effect: self.effect,
            delay: self.delay,
            skip: self.skip,
            skip_remaining: self.skip,
            count: self.count,
            remaining: self.count,
            repeat: self.repeat,
            message: self.message,
        })
    }
}

impl<P: ProbeCatalogEntry> ProbeInjectionSnapshot<P> {
    pub fn active_probe_keys(&self) -> Vec<&'static str> {
        self.scenarios
            .iter()
            .map(|scenario| scenario.probe.key())
            .collect()
    }
}

impl<P: ProbeCatalogEntry> ProbeInjectionStore<P> {
    pub fn new(config: ProbeInjectionConfig<P>) -> Self {
        Self { config }
    }

    pub fn from_raw(raw: &str) -> Self {
        Self::new(parse_probe_config(raw))
    }

    pub fn inject(&mut self, probe: P) -> Option<ClientTaskInjection> {
        let scenario = self
            .config
            .scenarios
            .iter_mut()
            .find(|scenario| scenario.probe == probe)?;

        eprintln!("[dev] calling probe: {:?}", &scenario.probe.key());

        if !scenario.should_fire() {
            return None;
        }

        eprintln!("[dev] firing probe: {:?}", &scenario.probe.key());

        match scenario.effect {
            ProbeEffect::Fail => Some(ClientTaskInjection::fail(
                probe.error(scenario.message.as_deref()),
                scenario.delay,
            )),
            ProbeEffect::DelayOnly => Some(ClientTaskInjection::delay_only(scenario.delay)),
        }
    }

    pub fn inject_by_name(&mut self, name: &str) -> Option<ClientTaskInjection> {
        let normalized_name = normalize_probe_name(name);
        let probe = P::ALL
            .iter()
            .copied()
            .find(|probe| probe.matches_name(&normalized_name))?;

        self.inject(probe)
    }

    pub fn snapshot(&self) -> ProbeInjectionSnapshot<P> {
        ProbeInjectionSnapshot {
            scenarios: self
                .config
                .scenarios
                .iter()
                .map(ProbeScenarioSnapshot::from)
                .collect(),
            unknown: self.config.unknown.clone(),
        }
    }

    pub fn clear(&mut self) {
        self.config = ProbeInjectionConfig {
            scenarios: Vec::new(),
            unknown: Vec::new(),
        };
    }

    pub fn set_probe_config(&mut self, probe: P, config: ProbeRuntimeConfig) {
        self.config
            .scenarios
            .retain(|scenario| scenario.probe != probe);

        if let Some(scenario) = config.into_scenario(probe) {
            self.config.scenarios.push(scenario);
        }
    }
}

impl<P: Copy> From<&ProbeScenarioConfig<P>> for ProbeScenarioSnapshot<P> {
    fn from(scenario: &ProbeScenarioConfig<P>) -> Self {
        Self {
            probe: scenario.probe,
            effect: scenario.effect,
            delay: scenario.delay,
            skip: scenario.skip,
            skip_remaining: scenario.skip_remaining,
            count: scenario.count,
            remaining: scenario.remaining,
            repeat: scenario.repeat,
            message: scenario.message.clone(),
        }
    }
}

pub fn parse_probe_config<P: ProbeCatalogEntry>(raw: &str) -> ProbeInjectionConfig<P> {
    let mut scenarios = Vec::new();
    let mut unknown = Vec::new();

    for token in raw.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }

        let (name_part, params_part) = match token.split_once(':') {
            Some((name, params)) => (name, Some(params)),
            None => (token, None),
        };

        let name = normalize_probe_name(name_part);
        if name.is_empty() {
            continue;
        }

        let params = parse_params(params_part);

        if name == "all" {
            for &probe in P::ALL {
                scenarios.push(scenario_from(probe, &params));
            }
            continue;
        }

        match P::ALL
            .iter()
            .copied()
            .find(|probe| probe.matches_name(&name))
        {
            Some(probe) => scenarios.push(scenario_from(probe, &params)),
            None => unknown.push(name_part.trim().to_string()),
        }
    }

    ProbeInjectionConfig { scenarios, unknown }
}

pub fn probe_catalog_items<P: ProbeCatalogEntry>() -> Vec<ProbeCatalogItem> {
    P::ALL
        .iter()
        .copied()
        .map(|probe| {
            let meta = probe.meta();
            ProbeCatalogItem {
                key: meta.key,
                short_key: meta.short_key,
                summary: meta.summary,
            }
        })
        .collect()
}

pub fn probe_catalog_keys<P: ProbeCatalogEntry>() -> Vec<&'static str> {
    P::ALL.iter().copied().map(ProbeCatalogEntry::key).collect()
}

pub fn probe_drafts_from_snapshot<P: ProbeCatalogEntry>(
    snapshot: &ProbeInjectionSnapshot<P>,
) -> Vec<ProbeDraft<P>> {
    P::ALL
        .iter()
        .copied()
        .map(|probe| {
            let scenario = snapshot
                .scenarios
                .iter()
                .find(|scenario| scenario.probe == probe);
            ProbeDraft::from_snapshot(probe, scenario)
        })
        .collect()
}

pub fn update_probe_drafts<P: Copy + Eq>(
    drafts: &mut [ProbeDraft<P>],
    message: ProbePanelMessage<P>,
) -> Option<ProbePanelEffect<P>> {
    match message {
        ProbePanelMessage::SetProbeEnabled(probe, enabled) => {
            update_probe_draft(drafts, probe, |draft| draft.set_enabled(enabled))
        }
        ProbePanelMessage::EffectChanged(probe, effect) => {
            update_probe_draft(drafts, probe, |draft| draft.set_effect(effect))
        }
        ProbePanelMessage::DelayChanged(probe, value) => {
            update_probe_draft(drafts, probe, |draft| draft.set_delay_input(value))
        }
        ProbePanelMessage::SkipChanged(probe, value) => {
            update_probe_draft(drafts, probe, |draft| draft.set_skip_input(value))
        }
        ProbePanelMessage::CountChanged(probe, value) => {
            update_probe_draft(drafts, probe, |draft| draft.set_count_input(value))
        }
        ProbePanelMessage::RepeatChanged(probe, repeat) => {
            update_probe_draft(drafts, probe, |draft| draft.set_repeat(repeat))
        }
        ProbePanelMessage::MessageChanged(probe, message) => {
            update_probe_draft(drafts, probe, |draft| draft.set_message(message))
        }
        ProbePanelMessage::FireNext(probe) => {
            update_probe_draft(drafts, probe, ProbeDraft::fire_next)
        }
        ProbePanelMessage::ClearProbe(probe) => {
            let draft = drafts.iter_mut().find(|draft| draft.probe == probe)?;
            *draft = ProbeDraft::disabled(probe);
            Some(ProbePanelEffect::SetProbeConfig(
                probe,
                ProbeRuntimeConfig::disabled(),
            ))
        }
        ProbePanelMessage::ClearAll => {
            for draft in drafts {
                *draft = ProbeDraft::disabled(draft.probe);
            }
            Some(ProbePanelEffect::ClearAll)
        }
    }
}

fn update_probe_draft<P: Copy + Eq>(
    drafts: &mut [ProbeDraft<P>],
    probe: P,
    update: impl FnOnce(&mut ProbeDraft<P>),
) -> Option<ProbePanelEffect<P>> {
    let draft = drafts.iter_mut().find(|draft| draft.probe == probe)?;

    update(draft);
    Some(ProbePanelEffect::SetProbeConfig(
        probe,
        draft.runtime_config(),
    ))
}

struct ParsedParams {
    effect: ProbeEffect,
    delay: Option<Duration>,
    skip: Option<u32>,
    count: Option<u32>,
    repeat: bool,
    message: Option<String>,
}

fn parse_params(params: Option<&str>) -> ParsedParams {
    let mut parsed = ParsedParams {
        effect: ProbeEffect::Fail,
        delay: None,
        skip: None,
        count: None,
        repeat: false,
        message: None,
    };

    let Some(params) = params else {
        return parsed;
    };

    for param in params.split(';') {
        let param = param.trim();
        if param.is_empty() {
            continue;
        }

        let (key, value) = match param.split_once('=') {
            Some((key, value)) => (key.trim(), Some(value.trim())),
            None => (param, None),
        };

        match key.to_ascii_lowercase().as_str() {
            "effect" | "mode" => {
                if let Some(value) = value {
                    parsed.effect = parse_effect(value);
                }
            }
            "delay_only" | "delay-only" => {
                parsed.effect = if value.map(parse_bool).unwrap_or(true) {
                    ProbeEffect::DelayOnly
                } else {
                    ProbeEffect::Fail
                };
            }
            "delay" => {
                if let Some(value) = value {
                    parsed.delay = parse_duration(value);
                }
            }
            "skip" => {
                if let Some(value) = value {
                    parsed.skip = value.parse().ok();
                }
            }
            "count" => {
                if let Some(value) = value {
                    parsed.count = value.parse().ok();
                }
            }
            "repeat" => parsed.repeat = value.map(parse_bool).unwrap_or(true),
            "message" | "msg" => {
                if let Some(value) = value {
                    parsed.message = Some(value.to_string());
                }
            }
            _ => {}
        }
    }

    parsed
}

fn scenario_from<P>(probe: P, params: &ParsedParams) -> ProbeScenarioConfig<P> {
    ProbeScenarioConfig {
        probe,
        effect: params.effect,
        delay: params.delay,
        skip: params.skip,
        skip_remaining: params.skip,
        count: params.count,
        remaining: params.count,
        repeat: params.repeat,
        message: params.message.clone(),
    }
}

fn parse_duration(value: &str) -> Option<Duration> {
    if let Some(millis) = value.strip_suffix("ms") {
        millis.trim().parse::<u64>().ok().map(Duration::from_millis)
    } else if let Some(secs) = value.strip_suffix('s') {
        secs.trim().parse::<u64>().ok().map(Duration::from_secs)
    } else {
        value.parse::<u64>().ok().map(Duration::from_millis)
    }
}

fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "on"
    )
}

fn parse_effect(value: &str) -> ProbeEffect {
    match normalize_probe_name(value).as_str() {
        "delay" | "delay_only" | "wait" => ProbeEffect::DelayOnly,
        _ => ProbeEffect::Fail,
    }
}

fn normalize_probe_name(token: &str) -> String {
    token.trim().replace('-', "_").to_ascii_lowercase()
}

fn decrement_if_positive(value: &mut Option<u32>) -> bool {
    match value {
        Some(remaining) if *remaining > 0 => {
            *remaining -= 1;
            true
        }
        Some(_) | None => false,
    }
}

fn numeric_input(value: String) -> String {
    value.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn parse_optional_u32(value: &str) -> Option<u32> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.parse().ok()).flatten()
}

fn parse_duration_ms(value: &str) -> Option<Duration> {
    parse_optional_u32(value).map(|millis| Duration::from_millis(u64::from(millis)))
}

fn duration_millis_string(duration: Duration) -> String {
    duration.as_millis().to_string()
}

fn text_matches_query<'a>(query: &str, values: impl IntoIterator<Item = &'a str>) -> bool {
    let query = query.trim().to_ascii_lowercase();
    query.is_empty()
        || values
            .into_iter()
            .any(|value| value.to_ascii_lowercase().contains(&query))
}

#[cfg(test)]
mod probe_tests {
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
        let config =
            parse_probe_config::<TestProbe>("create-project, tag.list:delay=2s;message=Boom");

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
        let mut store =
            ProbeInjectionStore::<TestProbe>::from_raw("tag.list:effect=delay;delay=250ms");

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
}
