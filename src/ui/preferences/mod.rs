use gtk::gio;
use gtk::prelude::*;
use libadwaita::prelude::*;

pub struct PreferencesWindow;

impl PreferencesWindow {
    pub fn new(settings: &gio::Settings) -> libadwaita::PreferencesWindow {
        let window = libadwaita::PreferencesWindow::builder()
            .title("Preferences")
            .build();

        let appearance = libadwaita::PreferencesGroup::builder()
            .title("Appearance")
            .build();

        let color_model = gtk::StringList::new(&["Follow System", "Light", "Dark"]);
        let color_row = libadwaita::ComboRow::builder()
            .title("Color Scheme")
            .model(&color_model)
            .build();

        color_row.set_selected(color_scheme_index(&settings.string("color-scheme")));

        let settings_for_color = settings.clone();
        color_row.connect_selected_notify(move |row| {
            let value = match row.selected() {
                1 => "light",
                2 => "dark",
                _ => "follow-system",
            };
            let _ = settings_for_color.set_string("color-scheme", value);
        });

        appearance.add(&color_row);

        let timestamps = libadwaita::PreferencesGroup::builder()
            .title("Timestamps")
            .build();

        let timestamp_enabled = libadwaita::SwitchRow::builder()
            .title("Auto-Add Timestamp")
            .build();
        settings
            .bind("timestamp-enabled", &timestamp_enabled, "active")
            .build();

        let timestamp_format = libadwaita::EntryRow::builder()
            .title("Timestamp Format")
            .build();
        settings
            .bind("timestamp-format", &timestamp_format, "text")
            .build();
        settings
            .bind("timestamp-enabled", &timestamp_format, "sensitive")
            .build();

        timestamps.add(&timestamp_enabled);
        timestamps.add(&timestamp_format);

        let automation = libadwaita::PreferencesGroup::builder()
            .title("Automation")
            .build();

        let auto_save = libadwaita::SwitchRow::builder().title("Auto Save").build();
        settings
            .bind("auto-save-enabled", &auto_save, "active")
            .build();

        let auto_export = libadwaita::SwitchRow::builder()
            .title("Auto Export")
            .build();
        settings
            .bind("auto-export-enabled", &auto_export, "active")
            .build();

        let export_suffix = libadwaita::EntryRow::builder()
            .title("Export Suffix")
            .build();
        settings
            .bind("auto-export-suffix", &export_suffix, "text")
            .build();
        settings
            .bind("auto-export-enabled", &export_suffix, "sensitive")
            .build();

        let auto_clipboard = libadwaita::SwitchRow::builder()
            .title("Auto Clipboard")
            .build();
        settings
            .bind("auto-clipboard-enabled", &auto_clipboard, "active")
            .build();

        automation.add(&auto_save);
        automation.add(&auto_export);
        automation.add(&export_suffix);
        automation.add(&auto_clipboard);

        let developer = libadwaita::PreferencesGroup::builder()
            .title("Developer")
            .build();

        let log_model = gtk::StringList::new(&["Error", "Warn", "Info", "Debug", "Trace"]);
        let log_row = libadwaita::ComboRow::builder()
            .title("Log Level")
            .model(&log_model)
            .build();

        log_row.set_selected(log_level_index(&settings.string("log-level")));

        let settings_for_log = settings.clone();
        log_row.connect_selected_notify(move |row| {
            let value = match row.selected() {
                0 => "error",
                1 => "warn",
                3 => "debug",
                4 => "trace",
                _ => "info",
            };
            let _ = settings_for_log.set_string("log-level", value);
        });

        developer.add(&log_row);

        let page = libadwaita::PreferencesPage::new();
        page.add(&appearance);
        page.add(&timestamps);
        page.add(&automation);
        page.add(&developer);
        window.add(&page);

        window
    }
}

fn color_scheme_index(value: &str) -> u32 {
    match value {
        "light" => 1,
        "dark" => 2,
        _ => 0,
    }
}

fn log_level_index(value: &str) -> u32 {
    match value {
        "error" => 0,
        "warn" => 1,
        "debug" => 3,
        "trace" => 4,
        _ => 2,
    }
}
