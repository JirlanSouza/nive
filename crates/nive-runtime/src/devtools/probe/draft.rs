use super::parse::{
    duration_millis_string, numeric_input, parse_duration_ms, parse_optional_u32,
    text_matches_query,
};
use super::{
    ProbeCatalogEntry, ProbeDraft, ProbeEffect, ProbeInjectionSnapshot, ProbePanelEffect,
    ProbePanelMessage, ProbeRuntimeConfig, ProbeScenarioSnapshot,
};

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

pub(super) fn update_probe_draft<P: Copy + Eq>(
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
