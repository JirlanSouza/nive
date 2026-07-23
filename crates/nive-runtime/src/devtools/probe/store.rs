use std::time::Duration;

use super::parse::{decrement_if_positive, normalize_probe_name, parse_probe_config};
use super::{
    ClientTaskInjection, ClientTaskInjectionKind, ProbeCatalogEntry, ProbeEffect,
    ProbeInjectionConfig, ProbeInjectionSnapshot, ProbeInjectionStore, ProbeRuntimeConfig,
    ProbeScenarioConfig, ProbeScenarioSnapshot,
};
use crate::UserFacingError;

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
