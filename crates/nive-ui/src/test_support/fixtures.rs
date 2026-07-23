use iced::{
    advanced::{clipboard, Clipboard},
    Rectangle, Size,
};

use super::{
    AnchoredGeometryFixture, FakeClock, FormStateFixture, MemoryClipboard, PopupStateFixture,
};

impl MemoryClipboard {
    pub(super) fn read_ref(&self, kind: clipboard::Kind) -> Option<&str> {
        match kind {
            clipboard::Kind::Standard => self.standard.as_deref(),
            clipboard::Kind::Primary => self.primary.as_deref(),
        }
    }
}

impl Clipboard for MemoryClipboard {
    fn read(&self, kind: clipboard::Kind) -> Option<String> {
        self.read_ref(kind).map(str::to_owned)
    }

    fn write(&mut self, kind: clipboard::Kind, contents: String) {
        match kind {
            clipboard::Kind::Standard => self.standard = Some(contents),
            clipboard::Kind::Primary => self.primary = Some(contents),
        }
    }
}

impl FakeClock {
    pub(crate) const fn at(now_ms: u64) -> Self {
        Self { now_ms }
    }

    pub(crate) const fn now_ms(self) -> u64 {
        self.now_ms
    }

    pub(crate) fn advance(&mut self, elapsed_ms: u64) {
        self.now_ms = self.now_ms.saturating_add(elapsed_ms);
    }
}

impl AnchoredGeometryFixture {
    pub(crate) const fn new(
        anchor: Rectangle,
        viewport: Rectangle,
        intrinsic_content: Size,
    ) -> Self {
        Self {
            anchor,
            viewport,
            intrinsic_content,
        }
    }
}

impl PopupStateFixture {
    pub(crate) const fn enabled() -> Self {
        Self {
            capable: true,
            disabled: false,
            open: false,
            selected: false,
            highlighted: false,
            focused: false,
            pressed: false,
        }
    }
}

impl FormStateFixture {
    pub(crate) const INTERACTIVE: [Self; 4] = [
        Self::enabled(),
        Self {
            hovered: true,
            ..Self::enabled()
        },
        Self {
            focused: true,
            ..Self::enabled()
        },
        Self {
            pressed: true,
            ..Self::enabled()
        },
    ];

    pub(crate) const fn enabled() -> Self {
        Self {
            hovered: false,
            focused: false,
            pressed: false,
            read_only: false,
            disabled: false,
        }
    }

    pub(crate) const fn read_only() -> Self {
        Self {
            read_only: true,
            ..Self::enabled()
        }
    }

    pub(crate) const fn disabled() -> Self {
        Self {
            disabled: true,
            ..Self::enabled()
        }
    }
}
