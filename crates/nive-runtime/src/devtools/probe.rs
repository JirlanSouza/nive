mod catalog;
mod draft;
mod parse;
mod store;

#[cfg(test)]
mod probe_tests;

pub use catalog::{composed_probe_ids, probe_catalog_items, probe_catalog_keys};
pub use draft::{probe_drafts_from_snapshot, update_probe_drafts};
pub use parse::parse_probe_config;

use std::time::Duration;

use crate::UserFacingError;

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
