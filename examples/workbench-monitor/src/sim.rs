use nive::prelude::ui::ToneRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Production,
    Staging,
}

#[derive(Debug, Clone)]
pub struct Service {
    pub id: &'static str,
    pub name: &'static str,
    pub host_id: &'static str,
    pub health: ToneRole,
    pub latency_ms: u32,
    pub uptime_percent: u32,
    pub requests_per_minute: u32,
    pub error_rate_percent: u32,
}

#[derive(Debug, Clone)]
pub struct Host {
    pub id: &'static str,
    pub name: &'static str,
    pub zone: &'static str,
    pub health: ToneRole,
    pub cpu_percent: u32,
    pub memory_percent: u32,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub id: u32,
    pub service_id: &'static str,
    pub title: &'static str,
    pub severity: ToneRole,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: u32,
    pub label: &'static str,
    pub progress: f32,
    pub running: bool,
}

#[derive(Debug, Clone)]
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
    pub fn seeded() -> Self {
        Self {
            tick: 0,
            environment: Environment::Production,
            services: vec![
                Service {
                    id: "api",
                    name: "API Gateway",
                    host_id: "edge-01",
                    health: ToneRole::Success,
                    latency_ms: 43,
                    uptime_percent: 100,
                    requests_per_minute: 18_420,
                    error_rate_percent: 0,
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
                },
            ],
            hosts: vec![
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
            ],
            alerts: vec![Alert {
                id: 1,
                service_id: "billing",
                title: "Billing queue latency above threshold",
                severity: ToneRole::Warning,
                active: true,
            }],
            logs: vec![
                "[0000] monitor connected to prod control plane".into(),
                "[0000] loaded 3 services and 3 hosts".into(),
            ],
            events: vec!["Initial service snapshot loaded".into()],
            jobs: Vec::new(),
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

    pub fn running_jobs(&self) -> usize {
        self.jobs.iter().filter(|job| job.running).count()
    }

    pub fn toggle_environment(&mut self) {
        self.environment = match self.environment {
            Environment::Production => Environment::Staging,
            Environment::Staging => Environment::Production,
        };
        self.events
            .push(format!("Switched environment to {}", self.environment_label()));
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
            id: self.tick as u32 + 100,
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

        if self.tick % 5 == 0 && self.alert(2).is_none() {
            self.alerts.push(Alert {
                id: 2,
                service_id: "api",
                title: "API p95 latency trend rising",
                severity: ToneRole::Info,
                active: true,
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
