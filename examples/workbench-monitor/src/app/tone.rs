use nive::prelude::*;

use crate::sim::Environment;

use super::WorkbenchMonitor;

impl WorkbenchMonitor {
    pub(super) fn active_alert_count(&self) -> usize {
        self.model.active_alerts().count()
    }

    pub(super) fn overall_tone(&self) -> ToneRole {
        if self
            .model
            .services
            .iter()
            .any(|service| service.health == ToneRole::Danger)
        {
            ToneRole::Danger
        } else if self
            .model
            .services
            .iter()
            .any(|service| service.health == ToneRole::Warning)
        {
            ToneRole::Warning
        } else {
            ToneRole::Success
        }
    }

    pub(super) fn host_tone(&self) -> ToneRole {
        if self
            .model
            .hosts
            .iter()
            .any(|host| host.health == ToneRole::Warning)
        {
            ToneRole::Warning
        } else {
            ToneRole::Success
        }
    }

    pub(super) fn alert_tone(&self) -> ToneRole {
        if self.active_alert_count() == 0 {
            ToneRole::Success
        } else {
            ToneRole::Warning
        }
    }

    pub(super) fn connection_tone(&self) -> ToneRole {
        match self.model.environment {
            Environment::Production => ToneRole::Success,
            Environment::Staging => ToneRole::Info,
        }
    }
}

pub(super) fn problem_severity(tone: ToneRole) -> ProblemSeverity {
    match tone {
        ToneRole::Danger => ProblemSeverity::Error,
        ToneRole::Warning => ProblemSeverity::Warning,
        ToneRole::Neutral | ToneRole::Accent | ToneRole::Info | ToneRole::Success => {
            ProblemSeverity::Info
        }
    }
}

pub(super) fn tone_label(tone: ToneRole) -> &'static str {
    match tone {
        ToneRole::Neutral => "neutral",
        ToneRole::Accent => "active",
        ToneRole::Info => "info",
        ToneRole::Success => "healthy",
        ToneRole::Warning => "warning",
        ToneRole::Danger => "critical",
    }
}
