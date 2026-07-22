use nive::prelude::ui::ToneRole;

/// Selects the Workbench Monitor's simulation source, resolved once at
/// startup from the documented `NIVE_MONITOR_FROZEN=1` environment value,
/// mirroring the `NIVE_DEVTOOLS` precedent. The mode is app-local: it is
/// never persisted and is not exposed as a `nive-workbench` feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationMode {
    /// The default interactive scenario, driven by the 900ms tick.
    Live,
    /// A deterministic fixture for review, with no tick subscription.
    Frozen,
}

impl SimulationMode {
    const ENV_VAR: &'static str = "NIVE_MONITOR_FROZEN";

    /// Resolves the mode from the process environment.
    pub fn from_env() -> Self {
        Self::from_env_value(std::env::var(Self::ENV_VAR).ok().as_deref())
    }

    fn from_env_value(value: Option<&str>) -> Self {
        match value {
            Some("1") => Self::Frozen,
            _ => Self::Live,
        }
    }

    /// Visible status-bar label for the active mode.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Frozen => "frozen",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Production,
    Staging,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Service {
    pub id: &'static str,
    pub name: &'static str,
    pub host_id: &'static str,
    pub health: ToneRole,
    pub latency_ms: u32,
    pub uptime_percent: u32,
    pub requests_per_minute: u32,
    pub error_rate_percent: u32,
    /// Other services this one calls, by id.
    pub dependencies: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Host {
    pub id: &'static str,
    pub name: &'static str,
    pub zone: &'static str,
    pub health: ToneRole,
    pub cpu_percent: u32,
    pub memory_percent: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Alert {
    pub id: u32,
    pub service_id: &'static str,
    pub title: &'static str,
    pub severity: ToneRole,
    pub active: bool,
    /// Visible relative age, e.g. "3m ago".
    pub age_label: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    pub id: u32,
    pub label: &'static str,
    pub progress: f32,
    pub running: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Simulation {
    pub tick: u64,
    pub environment: Environment,
    pub services: Vec<Service>,
    pub hosts: Vec<Host>,
    pub alerts: Vec<Alert>,
    pub logs: Vec<String>,
    pub events: Vec<String>,
    pub jobs: Vec<Job>,
}

impl Simulation {
    /// Realistic default services shared by every seed: a real deployment's
    /// relational and descriptive fields, including dependencies.
    fn seeded_services() -> Vec<Service> {
        vec![
            Service {
                id: "api",
                name: "API Gateway",
                host_id: "edge-01",
                health: ToneRole::Success,
                latency_ms: 43,
                uptime_percent: 100,
                requests_per_minute: 18_420,
                error_rate_percent: 0,
                dependencies: vec!["billing", "search"],
            },
            Service {
                id: "billing",
                name: "Billing Worker",
                host_id: "worker-02",
                health: ToneRole::Warning,
                latency_ms: 188,
                uptime_percent: 99,
                requests_per_minute: 3_840,
                error_rate_percent: 2,
                dependencies: vec!["api"],
            },
            Service {
                id: "search",
                name: "Search Index",
                host_id: "data-03",
                health: ToneRole::Success,
                latency_ms: 71,
                uptime_percent: 100,
                requests_per_minute: 9_210,
                error_rate_percent: 0,
                dependencies: vec!["api"],
            },
        ]
    }

    fn seeded_hosts() -> Vec<Host> {
        vec![
            Host {
                id: "edge-01",
                name: "edge-01",
                zone: "us-east-1a",
                health: ToneRole::Success,
                cpu_percent: 38,
                memory_percent: 54,
            },
            Host {
                id: "worker-02",
                name: "worker-02",
                zone: "us-east-1b",
                health: ToneRole::Warning,
                cpu_percent: 82,
                memory_percent: 73,
            },
            Host {
                id: "data-03",
                name: "data-03",
                zone: "us-east-1c",
                health: ToneRole::Success,
                cpu_percent: 44,
                memory_percent: 61,
            },
        ]
    }

    /// Log lines varying in severity, length, and message, with one line long
    /// enough to exercise the logs panel's overflow behavior.
    fn seeded_logs() -> Vec<String> {
        vec![
            "[0000] monitor connected to prod control plane".into(),
            "[0000] loaded 3 services and 3 hosts".into(),
            "[0001] billing: retrying stripe webhook delivery (attempt 2 of 5)".into(),
            "[0002] WARN api: connection pool at 82% utilization, consider scaling worker-02 \
             before the next traffic spike"
                .into(),
            "[0003] search: reindex completed in 4.2s, 128,402 documents processed".into(),
            "[0004] ERROR billing: payment gateway timeout after 30000ms, falling back to \
             queued retry"
                .into(),
        ]
    }

    /// Events including completed history, not only a live tail, with
    /// several naming a specific service so a document can scope to it.
    fn seeded_events() -> Vec<String> {
        vec![
            "Initial service snapshot loaded".into(),
            "Billing Worker deploy rev 9f31ad7c completed".into(),
            "API Gateway autoscaled to 6 instances".into(),
            "Search Index reindex completed".into(),
        ]
    }

    pub fn seeded() -> Self {
        Self {
            tick: 0,
            environment: Environment::Production,
            services: Self::seeded_services(),
            hosts: Self::seeded_hosts(),
            alerts: vec![
                Alert {
                    id: 1,
                    service_id: "billing",
                    title: "Billing queue latency above threshold",
                    severity: ToneRole::Warning,
                    active: true,
                    age_label: "3m ago",
                },
                Alert {
                    id: 3,
                    service_id: "api",
                    title: "API 5xx rate briefly exceeded 1%",
                    severity: ToneRole::Info,
                    active: false,
                    age_label: "51m ago",
                },
            ],
            logs: Self::seeded_logs(),
            events: Self::seeded_events(),
            jobs: vec![Job {
                id: 1,
                label: "Nightly health check",
                progress: 1.0,
                running: false,
            }],
        }
    }

    /// A deterministic fixture for the frozen review mode: both seeded
    /// alerts active at their distinct severities, one running and one
    /// complete job, and populated logs and events at a fixed tick — every
    /// reviewable content state is visible without waiting for a tick.
    pub fn frozen() -> Self {
        Self {
            tick: 7,
            environment: Environment::Production,
            services: Self::seeded_services(),
            hosts: Self::seeded_hosts(),
            alerts: vec![
                Alert {
                    id: 1,
                    service_id: "billing",
                    title: "Billing queue latency above threshold",
                    severity: ToneRole::Warning,
                    active: true,
                    age_label: "3m ago",
                },
                Alert {
                    id: 2,
                    service_id: "api",
                    title: "API p95 latency trend rising",
                    severity: ToneRole::Info,
                    active: true,
                    age_label: "12m ago",
                },
                Alert {
                    id: 3,
                    service_id: "search",
                    title: "Search Index cache miss rate spiked",
                    severity: ToneRole::Info,
                    active: false,
                    age_label: "51m ago",
                },
            ],
            logs: Self::seeded_logs(),
            events: Self::seeded_events(),
            jobs: vec![
                Job {
                    id: 1,
                    label: "Nightly health check",
                    progress: 1.0,
                    running: false,
                },
                Job {
                    id: 2,
                    label: "Run health check",
                    progress: 0.62,
                    running: true,
                },
            ],
        }
    }

    pub fn active_alerts(&self) -> impl Iterator<Item = &Alert> {
        self.alerts.iter().filter(|alert| alert.active)
    }

    pub fn service(&self, id: &str) -> Option<&Service> {
        self.services.iter().find(|service| service.id == id)
    }

    pub fn host(&self, id: &str) -> Option<&Host> {
        self.hosts.iter().find(|host| host.id == id)
    }

    pub fn alert(&self, id: u32) -> Option<&Alert> {
        self.alerts.iter().find(|alert| alert.id == id)
    }

    /// Resolves a service's dependencies to their current records.
    pub fn dependencies_of<'a>(&'a self, service: &Service) -> Vec<&'a Service> {
        service
            .dependencies
            .iter()
            .filter_map(|id| self.service(id))
            .collect()
    }

    /// Recent log lines and events naming this service, most recent first.
    pub fn recent_activity_for(&self, service: &Service) -> (Vec<&str>, Vec<&str>) {
        let logs = self
            .logs
            .iter()
            .rev()
            .filter(|line| line.contains(service.name) || line.contains(service.id))
            .map(String::as_str)
            .collect();
        let events = self
            .events
            .iter()
            .rev()
            .filter(|event| event.contains(service.name) || event.contains(service.id))
            .map(String::as_str)
            .collect();
        (logs, events)
    }

    pub fn running_jobs(&self) -> usize {
        self.jobs.iter().filter(|job| job.running).count()
    }

    pub fn toggle_environment(&mut self) {
        self.environment = match self.environment {
            Environment::Production => Environment::Staging,
            Environment::Staging => Environment::Production,
        };
        self.events.push(format!(
            "Switched environment to {}",
            self.environment_label()
        ));
    }

    pub const fn environment_label(&self) -> &'static str {
        match self.environment {
            Environment::Production => "prod",
            Environment::Staging => "staging",
        }
    }

    pub fn run_health_check(&mut self) {
        if self.jobs.iter().any(|job| job.running) {
            return;
        }

        self.jobs.push(Job {
            id: self.tick as u32 + 200,
            label: "Run health check",
            progress: 0.0,
            running: true,
        });
        self.events.push("Started fleet health check".into());
    }

    pub fn acknowledge_alert(&mut self, id: u32) {
        if let Some(alert) = self.alerts.iter_mut().find(|alert| alert.id == id) {
            alert.active = false;
            self.events.push(format!("Acknowledged alert {}", alert.id));
        }
    }

    pub fn advance(&mut self) -> bool {
        self.tick += 1;
        self.logs.push(format!(
            "[{:04}] sampled {} services in {}",
            self.tick,
            self.services.len(),
            self.environment_label()
        ));

        if self.tick.is_multiple_of(5) && self.alert(2).is_none() {
            self.alerts.push(Alert {
                id: 2,
                service_id: "api",
                title: "API p95 latency trend rising",
                severity: ToneRole::Info,
                active: true,
                age_label: "just now",
            });
            self.events.push("Raised API latency trend alert".into());
        }

        let mut completed = false;
        for job in &mut self.jobs {
            if job.running {
                job.progress = (job.progress + 0.18).min(1.0);
                if job.progress >= 1.0 {
                    job.running = false;
                    completed = true;
                }
            }
        }

        if completed {
            for service in &mut self.services {
                service.health = ToneRole::Success;
                service.error_rate_percent = 0;
                service.latency_ms = service.latency_ms.saturating_sub(24);
            }
            for host in &mut self.hosts {
                host.health = ToneRole::Success;
                host.cpu_percent = host.cpu_percent.saturating_sub(12);
            }
            self.events.push("Health check completed".into());
            self.logs
                .push(format!("[{:04}] health check completed", self.tick));
        }

        trim(&mut self.logs, 16);
        trim(&mut self.events, 12);

        completed
    }
}

fn trim(items: &mut Vec<String>, max: usize) {
    if items.len() > max {
        items.drain(0..items.len() - max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_mode_resolves_frozen_only_for_the_documented_value() {
        assert_eq!(SimulationMode::from_env_value(None), SimulationMode::Live);
        assert_eq!(
            SimulationMode::from_env_value(Some("")),
            SimulationMode::Live
        );
        assert_eq!(
            SimulationMode::from_env_value(Some("true")),
            SimulationMode::Live
        );
        assert_eq!(
            SimulationMode::from_env_value(Some("0")),
            SimulationMode::Live
        );
        assert_eq!(
            SimulationMode::from_env_value(Some("1")),
            SimulationMode::Frozen
        );
    }

    #[test]
    fn frozen_fixture_exposes_both_alert_severities_and_completed_and_running_jobs() {
        let sim = Simulation::frozen();

        let severities: Vec<_> = sim.active_alerts().map(|alert| alert.severity).collect();
        assert!(severities.contains(&ToneRole::Warning));
        assert!(severities.contains(&ToneRole::Info));

        assert_eq!(sim.running_jobs(), 1);
        assert!(sim
            .jobs
            .iter()
            .any(|job| !job.running && job.progress >= 1.0));
        assert!(sim.jobs.iter().any(|job| job.running && job.progress > 0.0));
    }

    #[test]
    fn two_frozen_seeds_are_equal() {
        assert_eq!(Simulation::frozen(), Simulation::frozen());
    }

    #[test]
    fn seeded_alerts_vary_in_severity_and_acknowledgement_state() {
        let sim = Simulation::seeded();

        assert!(sim.alerts.iter().any(|alert| alert.active));
        assert!(sim.alerts.iter().any(|alert| !alert.active));

        let first = sim.alerts[0].severity;
        assert!(sim.alerts.iter().any(|alert| alert.severity != first));
    }

    #[test]
    fn seeded_and_frozen_logs_are_not_degenerate_filler() {
        for sim in [Simulation::seeded(), Simulation::frozen()] {
            let unique: std::collections::HashSet<_> = sim.logs.iter().collect();
            assert!(unique.len() > 1, "log lines must not all repeat");
            assert!(
                sim.logs.iter().any(|line| line.len() > 80),
                "at least one log line should be long enough to exercise overflow"
            );
        }
    }

    #[test]
    fn seeded_and_frozen_events_and_jobs_include_completed_history() {
        for sim in [Simulation::seeded(), Simulation::frozen()] {
            assert!(sim.events.len() > 1);
            assert!(
                sim.jobs
                    .iter()
                    .any(|job| !job.running && job.progress >= 1.0),
                "seed should include a completed job, not only a live tail"
            );
        }
    }

    #[test]
    fn dependencies_of_resolves_to_existing_services() {
        let sim = Simulation::seeded();
        let api = sim.service("api").expect("seeded api service");

        let deps = sim.dependencies_of(api);

        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|service| service.id == "billing"));
        assert!(deps.iter().any(|service| service.id == "search"));
    }

    #[test]
    fn recent_activity_for_scopes_to_the_named_service() {
        let sim = Simulation::seeded();
        let billing = sim.service("billing").expect("seeded billing service");

        let (logs, events) = sim.recent_activity_for(billing);

        assert!(logs.iter().any(|line| line.contains("billing")));
        assert!(events.iter().any(|event| event.contains("Billing Worker")));
    }
}
