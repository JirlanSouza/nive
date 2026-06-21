use serde::{Deserialize, Serialize};

use iced::{Point, Size};

use crate::ThemePreference;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RuntimeSession {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "theme_preference"
    )]
    theme_preference: Option<ThemePreference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    windows: Vec<WindowSession>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowSession {
    key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    size: Option<WindowSessionSize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    position: Option<WindowSessionPosition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowSessionSize {
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowSessionPosition {
    x: f32,
    y: f32,
}

impl RuntimeSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn theme_preference(&self) -> Option<ThemePreference> {
        self.theme_preference
    }

    pub fn set_theme_preference(&mut self, preference: Option<ThemePreference>) {
        self.theme_preference = preference;
    }

    pub fn with_theme_preference(mut self, preference: ThemePreference) -> Self {
        self.set_theme_preference(Some(preference));
        self
    }

    pub fn windows(&self) -> &[WindowSession] {
        self.windows.as_slice()
    }

    pub fn window(&self, key: &str) -> Option<&WindowSession> {
        self.windows.iter().find(|window| window.key() == key)
    }

    pub fn upsert_window(&mut self, window: WindowSession) {
        if let Some(existing) = self
            .windows
            .iter_mut()
            .find(|existing| existing.key == window.key)
        {
            *existing = window;
        } else {
            self.windows.push(window);
        }
    }

    pub fn with_window(mut self, window: WindowSession) -> Self {
        self.upsert_window(window);
        self
    }

    pub(crate) fn set_window_size(&mut self, key: &str, size: Size) -> bool {
        let window = self.window_or_insert(key);
        window.set_size(size)
    }

    pub(crate) fn set_window_position(&mut self, key: &str, position: Point) -> bool {
        let window = self.window_or_insert(key);
        window.set_position(position)
    }

    fn window_or_insert(&mut self, key: &str) -> &mut WindowSession {
        if let Some(index) = self.windows.iter().position(|window| window.key() == key) {
            &mut self.windows[index]
        } else {
            self.windows.push(WindowSession::new(key));
            self.windows
                .last_mut()
                .expect("window session inserted into non-empty vec")
        }
    }
}

impl WindowSession {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            size: None,
            position: None,
        }
    }

    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    pub fn size(&self) -> Option<Size> {
        self.size.and_then(WindowSessionSize::to_size)
    }

    pub fn position(&self) -> Option<Point> {
        self.position.and_then(WindowSessionPosition::to_point)
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Some(WindowSessionSize::new(width, height));
        self
    }

    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = Some(WindowSessionPosition::new(x, y));
        self
    }

    fn set_size(&mut self, size: Size) -> bool {
        let next = WindowSessionSize::from_size(size);
        if self.size == Some(next) {
            false
        } else {
            self.size = Some(next);
            true
        }
    }

    fn set_position(&mut self, position: Point) -> bool {
        let next = WindowSessionPosition::from_point(position);
        if self.position == Some(next) {
            false
        } else {
            self.position = Some(next);
            true
        }
    }
}

impl WindowSessionSize {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn width(self) -> f32 {
        self.width
    }

    pub fn height(self) -> f32 {
        self.height
    }

    fn from_size(size: Size) -> Self {
        Self::new(size.width, size.height)
    }

    fn to_size(self) -> Option<Size> {
        (self.width.is_finite() && self.height.is_finite() && self.width > 0.0 && self.height > 0.0)
            .then(|| Size::new(self.width, self.height))
    }
}

impl WindowSessionPosition {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn x(self) -> f32 {
        self.x
    }

    pub fn y(self) -> f32 {
        self.y
    }

    fn from_point(point: Point) -> Self {
        Self::new(point.x, point.y)
    }

    fn to_point(self) -> Option<Point> {
        (self.x.is_finite() && self.y.is_finite()).then(|| Point::new(self.x, self.y))
    }
}

mod theme_preference {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::ThemePreference;

    pub fn serialize<S>(
        preference: &Option<ThemePreference>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        preference.map(as_str).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ThemePreference>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Some(value) = Option::<String>::deserialize(deserializer)? else {
            return Ok(None);
        };

        match value.as_str() {
            "system" => Ok(Some(ThemePreference::System)),
            "light" => Ok(Some(ThemePreference::Light)),
            "dark" => Ok(Some(ThemePreference::Dark)),
            _ => Err(serde::de::Error::custom(format!(
                "unknown theme preference: {value}"
            ))),
        }
    }

    fn as_str(preference: ThemePreference) -> &'static str {
        match preference {
            ThemePreference::System => "system",
            ThemePreference::Light => "light",
            ThemePreference::Dark => "dark",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_theme_preference_as_stable_string() {
        let session = RuntimeSession::new().with_theme_preference(ThemePreference::Dark);
        let json = serde_json::to_string(&session).expect("session should serialize");

        assert!(json.contains("\"theme_preference\":\"dark\""));
    }

    #[test]
    fn deserializes_theme_preference_from_stable_string() {
        let session: RuntimeSession =
            serde_json::from_str(r#"{"theme_preference":"light"}"#).expect("valid session json");

        assert_eq!(session.theme_preference(), Some(ThemePreference::Light));
    }

    #[test]
    fn serializes_window_geometry_as_stable_objects() {
        let session = RuntimeSession::new().with_window(
            WindowSession::new("workspace")
                .with_size(1280.0, 820.0)
                .with_position(120.0, 80.0),
        );
        let json = serde_json::to_string(&session).expect("session should serialize");

        assert!(json.contains("\"key\":\"workspace\""));
        assert!(json.contains("\"size\":{\"width\":1280.0,\"height\":820.0}"));
        assert!(json.contains("\"position\":{\"x\":120.0,\"y\":80.0}"));
    }

    #[test]
    fn upserts_window_session_by_stable_key() {
        let mut session = RuntimeSession::new()
            .with_window(WindowSession::new("workspace").with_size(1024.0, 720.0));

        session.upsert_window(WindowSession::new("workspace").with_size(1280.0, 820.0));

        assert_eq!(session.windows().len(), 1);
        assert_eq!(
            session.window("workspace").and_then(WindowSession::size),
            Some(Size::new(1280.0, 820.0))
        );
    }

    #[test]
    fn ignores_invalid_persisted_window_geometry() {
        let session: RuntimeSession = serde_json::from_str(
            r#"{"windows":[{"key":"workspace","size":{"width":0.0,"height":820.0},"position":{"x":120.0,"y":80.0}}]}"#,
        )
        .expect("session json should deserialize");
        let window = session
            .window("workspace")
            .expect("window session should exist");

        assert_eq!(window.size(), None);
        assert_eq!(window.position(), Some(Point::new(120.0, 80.0)));
    }
}
