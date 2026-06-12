use gtk::gio;
use gtk::prelude::*;
use log::LevelFilter;

const SCHEMA_ID: &str = "com.screenshot_hero.ScreenshotHero";
const SCHEMA_PATH: &str = "/com/screenshot_hero/ScreenshotHero/";

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
