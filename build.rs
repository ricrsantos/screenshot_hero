use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    compile_gsettings(&manifest_dir);
    compile_gresources(&manifest_dir);
}

fn compile_gsettings(manifest_dir: &Path) {
    let schema_xml = manifest_dir.join("data/dev.codethings.schero.gschema.xml");
    if !schema_xml.exists() {
        return;
    }

    println!("cargo:rerun-if-changed={}", schema_xml.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let schema_dir = out_dir.join("schemas");
    fs::create_dir_all(&schema_dir).expect("create schema output directory");

    let installed_xml = schema_dir.join("dev.codethings.schero.gschema.xml");
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

fn compile_gresources(manifest_dir: &Path) {
    let resource_xml = manifest_dir.join("data/dev.codethings.schero.gresource.xml");
    if !resource_xml.exists() {
        return;
    }

    println!("cargo:rerun-if-changed={}", resource_xml.display());
    emit_rerun_if_changed_for_dir(&manifest_dir.join("data/resources/icons"));
    emit_rerun_if_changed_for_dir(&manifest_dir.join("data/icons/hicolor"));

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let output_resource = out_dir.join("dev.codethings.schero.gresource");

    let compiled = Command::new("glib-compile-resources")
        .arg(&resource_xml)
        .arg("--sourcedir")
        .arg(manifest_dir.join("data"))
        .arg("--target")
        .arg(&output_resource)
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    if compiled {
        println!(
            "cargo:rustc-env=APP_GRESOURCE_PATH={}",
            output_resource.display()
        );
    } else {
        panic!(
            "glib-compile-resources is required to embed SVG/PNG assets via GResource"
        );
    }
}

fn emit_rerun_if_changed_for_dir(dir: &Path) {
    if !dir.exists() {
        return;
    }

    println!("cargo:rerun-if-changed={}", dir.display());

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            emit_rerun_if_changed_for_dir(&path);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
