use gtk::gio;
use gtk::prelude::*;
use log::LevelFilter;
use std::time::{SystemTime, UNIX_EPOCH};

const SCHEMA_ID: &str = "dev.codethings.schero";
const SCHEMA_PATH: &str = "/dev/codethings/schero/";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColorSchemePreference {
    FollowSystem,
    Light,
    Dark,
}

impl ColorSchemePreference {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FollowSystem => "follow-system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::FollowSystem,
        }
    }

    pub fn to_adw_color_scheme(&self) -> libadwaita::ColorScheme {
        match self {
            Self::FollowSystem => libadwaita::ColorScheme::Default,
            Self::Light => libadwaita::ColorScheme::ForceLight,
            Self::Dark => libadwaita::ColorScheme::ForceDark,
        }
    }
}

#[derive(Clone)]
pub struct AppSettings {
    inner: gio::Settings,
}

impl AppSettings {
    pub fn try_new() -> Option<Self> {
        if gio::SettingsSchemaSource::default()
            .and_then(|source| source.lookup(SCHEMA_ID, true))
            .is_some()
        {
            return Some(Self::new());
        }

        if let Some(dir) = option_env!("APP_GSETTINGS_SCHEMA_DIR") {
            if let Some(settings) = Self::try_from_directory(dir) {
                return Some(settings);
            }
        }

        None
    }

    fn try_from_directory(dir: &str) -> Option<Self> {
        let parent = gio::SettingsSchemaSource::default();
        let source = gio::SettingsSchemaSource::from_directory(dir, parent.as_ref(), true).ok()?;
        let schema = source.lookup(SCHEMA_ID, true)?;
        Some(Self {
            inner: gio::Settings::new_full(&schema, None::<&gio::SettingsBackend>, None),
        })
    }

    pub fn new() -> Self {
        Self {
            inner: gio::Settings::with_path(SCHEMA_ID, SCHEMA_PATH),
        }
    }

    pub fn settings(&self) -> &gio::Settings {
        &self.inner
    }

    pub fn color_scheme(&self) -> ColorSchemePreference {
        ColorSchemePreference::from_str(&self.inner.string("color-scheme"))
    }

    pub fn set_color_scheme(&self, value: ColorSchemePreference) {
        let _ = self.inner.set_string("color-scheme", value.as_str());
    }

    pub fn timestamp_enabled(&self) -> bool {
        self.inner.boolean("timestamp-enabled")
    }

    pub fn set_timestamp_enabled(&self, value: bool) {
        let _ = self.inner.set_boolean("timestamp-enabled", value);
    }

    pub fn timestamp_format(&self) -> String {
        self.inner.string("timestamp-format").to_string()
    }

    pub fn set_timestamp_format(&self, value: &str) {
        let _ = self.inner.set_string("timestamp-format", value);
    }

    pub fn auto_save_enabled(&self) -> bool {
        self.inner.boolean("auto-save-enabled")
    }

    pub fn set_auto_save_enabled(&self, value: bool) {
        let _ = self.inner.set_boolean("auto-save-enabled", value);
    }

    pub fn auto_export_enabled(&self) -> bool {
        self.inner.boolean("auto-export-enabled")
    }

    pub fn set_auto_export_enabled(&self, value: bool) {
        let _ = self.inner.set_boolean("auto-export-enabled", value);
    }

    pub fn auto_export_suffix(&self) -> String {
        self.inner.string("auto-export-suffix").to_string()
    }

    pub fn set_auto_export_suffix(&self, value: &str) {
        let _ = self.inner.set_string("auto-export-suffix", value);
    }

    pub fn auto_clipboard_enabled(&self) -> bool {
        self.inner.boolean("auto-clipboard-enabled")
    }

    pub fn set_auto_clipboard_enabled(&self, value: bool) {
        let _ = self.inner.set_boolean("auto-clipboard-enabled", value);
    }

    pub fn post_capture_editing_disabled(&self) -> bool {
        self.inner.boolean("post-capture-editing-disabled")
    }

    pub fn set_post_capture_editing_disabled(&self, value: bool) {
        let _ = self
            .inner
            .set_boolean("post-capture-editing-disabled", value);
    }

    pub fn post_capture_editing_temporary_enabled(&self) -> bool {
        self.inner.boolean("post-capture-editing-temporary-enabled")
    }

    pub fn set_post_capture_editing_temporary_enabled(&self, value: bool) {
        let _ = self
            .inner
            .set_boolean("post-capture-editing-temporary-enabled", value);
        if value {
            self.set_post_capture_editing_temporary_started_at(current_epoch_seconds());
        } else {
            self.set_post_capture_editing_temporary_started_at(0);
        }
    }

    pub fn post_capture_editing_temporary_minutes(&self) -> u32 {
        self.inner.uint("post-capture-editing-temporary-minutes")
    }

    pub fn set_post_capture_editing_temporary_minutes(&self, value: u32) {
        let _ = self
            .inner
            .set_uint("post-capture-editing-temporary-minutes", value);
    }

    pub fn post_capture_editing_temporary_seconds(&self) -> u32 {
        self.inner.uint("post-capture-editing-temporary-seconds")
    }

    pub fn set_post_capture_editing_temporary_seconds(&self, value: u32) {
        let _ = self
            .inner
            .set_uint("post-capture-editing-temporary-seconds", value);
    }

    pub fn post_capture_editing_temporary_started_at(&self) -> i64 {
        self.inner
            .int64("post-capture-editing-temporary-started-at")
    }

    pub fn set_post_capture_editing_temporary_started_at(&self, value: i64) {
        let _ = self
            .inner
            .set_int64("post-capture-editing-temporary-started-at", value);
    }

    pub fn open_new_window_on_capture(&self) -> bool {
        self.inner.boolean("open-new-window-on-capture")
    }

    pub fn set_open_new_window_on_capture(&self, value: bool) {
        let _ = self.inner.set_boolean("open-new-window-on-capture", value);
    }

    pub fn is_post_capture_editing_effectively_disabled(&self) -> bool {
        if self.post_capture_editing_disabled() {
            return true;
        }

        if !self.post_capture_editing_temporary_enabled() {
            return false;
        }

        let duration = i64::from(self.post_capture_editing_temporary_minutes()) * 60
            + i64::from(self.post_capture_editing_temporary_seconds());
        let duration = duration.max(1);
        let now = current_epoch_seconds();
        let started_at = self.post_capture_editing_temporary_started_at();
        let started_at = if started_at <= 0 {
            let _ = self
                .inner
                .set_int64("post-capture-editing-temporary-started-at", now);
            now
        } else {
            started_at
        };

        if now - started_at < duration {
            true
        } else {
            let _ = self
                .inner
                .set_boolean("post-capture-editing-temporary-enabled", false);
            let _ = self
                .inner
                .set_int64("post-capture-editing-temporary-started-at", 0);
            false
        }
    }

    pub fn log_level(&self) -> LevelFilter {
        match self.inner.string("log-level").as_str() {
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Info,
        }
    }

    pub fn set_log_level(&self, value: LevelFilter) {
        let level = match value {
            LevelFilter::Error => "error",
            LevelFilter::Warn => "warn",
            LevelFilter::Info => "info",
            LevelFilter::Debug => "debug",
            LevelFilter::Trace => "trace",
            LevelFilter::Off => "info",
        };
        let _ = self.inner.set_string("log-level", level);
    }

    pub fn connect_changed<F: Fn(&str) + 'static>(&self, f: F) {
        self.inner.connect_changed(None, move |_, key| {
            f(key);
        });
    }
}

fn current_epoch_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}
