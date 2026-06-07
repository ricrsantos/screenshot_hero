use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let schema_xml = manifest_dir.join("data/com.screenshot_hero.ScreenshotHero.gschema.xml");

    if !schema_xml.exists() {
        return;
    }

    println!("cargo:rerun-if-changed={}", schema_xml.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let schema_dir = out_dir.join("schemas");
    fs::create_dir_all(&schema_dir).expect("create schema output directory");

    let installed_xml = schema_dir.join("com.screenshot_hero.ScreenshotHero.gschema.xml");
    fs::copy(&schema_xml, &installed_xml).expect("copy gschema.xml into build output");

    let compiled = Command::new("glib-compile-schemas")
        .arg(&schema_dir)
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    if compiled {
        println!(
            "cargo:rustc-env=APP_GSETTINGS_SCHEMA_DIR={}",
            schema_dir.display()
        );
    } else {
        println!(
            "cargo:warning=glib-compile-schemas not available; \
             run `glib-compile-schemas data/` for local GSettings support"
        );
        let dev_dir = manifest_dir.join("data");
        println!(
            "cargo:rustc-env=APP_GSETTINGS_SCHEMA_DIR={}",
            dev_dir.display()
        );
    }
}
