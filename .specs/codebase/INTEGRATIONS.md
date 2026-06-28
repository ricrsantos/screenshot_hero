# External Integrations

## Desktop Portal Integration

**Service:** XDG Desktop Portal (Screenshot API via `ashpd`)  
**Purpose:** Capture screen regions through desktop-native permissioned flow  
**Implementation:** `src/capture/service.rs` using `ashpd::desktop::screenshot::Screenshot`  
**Configuration:** Flatpak `finish-args` include portal talk names in `build/com.screenshot_hero.ScreenshotHero.yml`  
**Authentication:** User-mediated portal permission prompts (desktop-level)

## Filesystem Integration

**Service:** Local filesystem (images, project files, exports)  
**Purpose:** Open source images, persist `.shero` projects, write exports  
**Implementation:** `src/capture/loader.rs`, `src/persistence/*.rs`, `src/export/exporter.rs`  
**Configuration:** Flatpak grants read access to `xdg-pictures` and portal-mediated save/open paths  
**Authentication:** OS filesystem permissions and portal-selected handles

## Clipboard Integration

**Service:** GDK clipboard  
**Purpose:** Copy rendered annotated images for paste workflows  
**Implementation:** `src/export/clipboard.rs` and `copy_to_clipboard` action wiring in `src/ui/window/imp.rs`  
**Configuration:** Requires active display session  
**Authentication:** Session/display-level access (no external credentials)

## Settings Integration

**Service:** GSettings (`gio::Settings`)  
**Purpose:** Persist user preferences (theme, automation flags, log level, timestamp behavior)  
**Implementation:** `src/settings/mod.rs` + schema in `data/com.screenshot_hero.ScreenshotHero.gschema.xml`  
**Configuration:** Schema compiled locally and in Flatpak build (`glib-compile-schemas`)  
**Authentication:** None (local settings store)

## Resource/Theming Integration

**Service:** GResource and icon theme infrastructure  
**Purpose:** Bundle and expose app icons/resources to GTK runtime  
**Implementation:** `src/resources.rs`, `data/com.screenshot_hero.ScreenshotHero.gresource.xml`  
**Configuration:** Resource registration at startup and icon theme path injection  
**Authentication:** None

## API/Webhooks/Background Jobs

- API integrations: none (offline desktop app)
- Webhooks: none
- Queue/background job infrastructure: none (only GLib main-loop callbacks/timeouts)
