use gtk::gio;
use gtk::prelude::*;
use libadwaita::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

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

        let capture_behavior = libadwaita::PreferencesGroup::builder()
            .title("Capture Behavior")
            .description("Options applied when Screenshot Hero runs with --capture")
            .build();

        let disable_post_capture = libadwaita::SwitchRow::builder()
            .title("Disable Post-Capture Editing")
            .subtitle("Capture with GNOME and exit without opening editor")
            .build();
        settings
            .bind(
                "post-capture-editing-disabled",
                &disable_post_capture,
                "active",
            )
            .build();

        let temporary_disable_post_capture = libadwaita::SwitchRow::builder()
            .title("Temporary Disable Post-Capture Editing")
            .subtitle("Keep capture-and-exit active for a limited time")
            .build();
        settings
            .bind(
                "post-capture-editing-temporary-enabled",
                &temporary_disable_post_capture,
                "active",
            )
            .build();

        let settings_for_temporary = settings.clone();
        temporary_disable_post_capture.connect_active_notify(move |row| {
            if row.is_active() {
                let _ = settings_for_temporary.set_int64(
                    "post-capture-editing-temporary-started-at",
                    current_epoch_seconds(),
                );
            } else {
                let _ = settings_for_temporary
                    .set_int64("post-capture-editing-temporary-started-at", 0);
            }
        });

        let temporary_minutes_row = libadwaita::ActionRow::builder()
            .title("Temporary Duration (Minutes)")
            .build();
        let temporary_minutes = gtk::SpinButton::with_range(0.0, 240.0, 1.0);
        temporary_minutes.set_valign(gtk::Align::Center);
        temporary_minutes.set_numeric(true);
        temporary_minutes.set_value(settings.uint("post-capture-editing-temporary-minutes") as f64);
        let settings_for_minutes = settings.clone();
        temporary_minutes.connect_value_changed(move |spin| {
            let value = spin.value_as_int().max(0) as u32;
            let _ = settings_for_minutes.set_uint("post-capture-editing-temporary-minutes", value);
        });
        temporary_minutes_row.add_suffix(&temporary_minutes);

        let temporary_seconds_row = libadwaita::ActionRow::builder()
            .title("Temporary Duration (Seconds)")
            .build();
        let temporary_seconds = gtk::SpinButton::with_range(0.0, 59.0, 1.0);
        temporary_seconds.set_valign(gtk::Align::Center);
        temporary_seconds.set_numeric(true);
        temporary_seconds.set_value(settings.uint("post-capture-editing-temporary-seconds") as f64);
        let settings_for_seconds = settings.clone();
        temporary_seconds.connect_value_changed(move |spin| {
            let value = spin.value_as_int().max(0) as u32;
            let _ = settings_for_seconds.set_uint("post-capture-editing-temporary-seconds", value);
        });
        temporary_seconds_row.add_suffix(&temporary_seconds);

        let temporary_enabled = settings.boolean("post-capture-editing-temporary-enabled");
        temporary_minutes_row.set_sensitive(temporary_enabled);
        temporary_seconds_row.set_sensitive(temporary_enabled);
        let minutes_row_for_changed = temporary_minutes_row.clone();
        let seconds_row_for_changed = temporary_seconds_row.clone();
        settings.connect_changed(
            Some("post-capture-editing-temporary-enabled"),
            move |s, _| {
                let enabled = s.boolean("post-capture-editing-temporary-enabled");
                minutes_row_for_changed.set_sensitive(enabled);
                seconds_row_for_changed.set_sensitive(enabled);
            },
        );

        let open_new_window = libadwaita::SwitchRow::builder()
            .title("Open New Window on Capture")
            .subtitle("When disabled, replace the current image")
            .build();
        settings
            .bind("open-new-window-on-capture", &open_new_window, "active")
            .build();

        let exit_after_paste = libadwaita::SwitchRow::builder()
            .title("Exit After Paste")
            .subtitle("Quit after captured image leaves clipboard")
            .build();
        settings
            .bind("exit-after-paste", &exit_after_paste, "active")
            .build();

        capture_behavior.add(&disable_post_capture);
        capture_behavior.add(&temporary_disable_post_capture);
        capture_behavior.add(&temporary_minutes_row);
        capture_behavior.add(&temporary_seconds_row);
        capture_behavior.add(&open_new_window);
        capture_behavior.add(&exit_after_paste);

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
        page.add(&capture_behavior);
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

fn current_epoch_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}
