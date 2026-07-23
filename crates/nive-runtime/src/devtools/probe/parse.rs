use std::time::Duration;

use super::{ProbeCatalogEntry, ProbeEffect, ProbeInjectionConfig, ProbeScenarioConfig};

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

pub(super) fn parse_duration(value: &str) -> Option<Duration> {
    if let Some(millis) = value.strip_suffix("ms") {
        millis.trim().parse::<u64>().ok().map(Duration::from_millis)
    } else if let Some(secs) = value.strip_suffix('s') {
        secs.trim().parse::<u64>().ok().map(Duration::from_secs)
    } else {
        value.parse::<u64>().ok().map(Duration::from_millis)
    }
}

pub(super) fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "on"
    )
}

pub(super) fn parse_effect(value: &str) -> ProbeEffect {
    match normalize_probe_name(value).as_str() {
        "delay" | "delay_only" | "wait" => ProbeEffect::DelayOnly,
        _ => ProbeEffect::Fail,
    }
}

pub(super) fn normalize_probe_name(token: &str) -> String {
    token.trim().replace('-', "_").to_ascii_lowercase()
}

pub(super) fn decrement_if_positive(value: &mut Option<u32>) -> bool {
    match value {
        Some(remaining) if *remaining > 0 => {
            *remaining -= 1;
            true
        }
        Some(_) | None => false,
    }
}

pub(super) fn numeric_input(value: String) -> String {
    value.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

pub(super) fn parse_optional_u32(value: &str) -> Option<u32> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.parse().ok()).flatten()
}

pub(super) fn parse_duration_ms(value: &str) -> Option<Duration> {
    parse_optional_u32(value).map(|millis| Duration::from_millis(u64::from(millis)))
}

pub(super) fn duration_millis_string(duration: Duration) -> String {
    duration.as_millis().to_string()
}

pub(super) fn text_matches_query<'a>(
    query: &str,
    values: impl IntoIterator<Item = &'a str>,
) -> bool {
    let query = query.trim().to_ascii_lowercase();
    query.is_empty()
        || values
            .into_iter()
            .any(|value| value.to_ascii_lowercase().contains(&query))
}
