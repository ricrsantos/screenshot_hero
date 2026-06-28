# Flathub Submission Checklist

This checklist is for submitting `Screenshot Hero` to Flathub with app id `dev.codethings.schero`.

## 1) Validate metadata locally

Run from project root:

```bash
desktop-file-validate data/dev.codethings.schero.desktop
appstreamcli validate --pedantic data/dev.codethings.schero.metainfo.xml
```

Expected: no errors and no remaining actionable warnings.

## 2) Build-test local Flatpak manifest

```bash
flatpak-builder --build-only --force-clean flatpak_artifacts/flathub-check-build flatpak/dev.codethings.schero.yml
```

Optional runtime test:

```bash
flatpak-builder --user --install --force-clean flatpak_artifacts/flathub-check-install flatpak/dev.codethings.schero.yml
flatpak run dev.codethings.schero
```

## 3) Refresh Cargo sources lock for release

When dependencies or lockfile change, regenerate `flatpak/cargo-sources.json` before opening/updating the Flathub PR.

Common command:

```bash
flatpak-cargo-generator -o flatpak/cargo-sources.json Cargo.lock
```

## 4) Prepare Flathub PR files

In your `flathub/flathub` fork:

- Copy `flatpak/dev.codethings.schero.flathub.yml` to repository root as `dev.codethings.schero.yml`
- Copy `flatpak/cargo-sources.json` to repository root as `cargo-sources.json`
- Confirm the `commit` field in the manifest points to an upstream commit that already includes the new `dev.codethings.schero` data files

## 5) Open pull request on flathub/flathub

In the PR description, include:

- What the app does (`Screenshot Hero`)
- Why requested permissions are needed
- Upstream URL: <https://github.com/ricrsantos/screenshot_hero>
- Basic test steps for reviewers

## 6) Follow CI and reviewer feedback

- Address lint/build feedback from Flathub CI
- If commit changes upstream, update manifest `commit` and `cargo-sources.json`
- Re-run local checks from steps 1 and 2 before each update

## 7) After merge

Confirm install from Flathub:

```bash
flatpak install flathub dev.codethings.schero
flatpak run dev.codethings.schero
```
