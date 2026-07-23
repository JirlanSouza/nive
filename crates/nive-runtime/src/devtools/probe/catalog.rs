use super::parse::normalize_probe_name;
use super::{
    ComposedProbeId, ProbeCatalogEntry, ProbeCatalogItem, ProbeErrorScope, ProbeMeta,
    ProbeMetaCatalog, ProbeMetaCatalogIter,
};
use crate::UserFacingError;

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
