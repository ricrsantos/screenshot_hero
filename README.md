<p align="center">
  <a href="#english">🇺🇸 English</a> |
  <a href="#portugues-br">🇧🇷 Português</a>
</p>

<h1 align="center">Screenshot Hero</h1>

<p align="center">
  Capture, annotate, and share screenshots on Linux. Fully compatible with GNOME and Wayland.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20GNOME-blue" alt="Linux GNOME">
  <img src="https://img.shields.io/badge/display-Wayland-purple" alt="Wayland">
  <img src="https://img.shields.io/badge/built%20with-Rust%20%7C%20GTK4%20%7C%20Libadwaita-orange" alt="Rust GTK4 Libadwaita">
  <img src="https://img.shields.io/badge/distribution-Flatpak-4A86CF" alt="Flatpak">
  <img src="https://img.shields.io/badge/license-BSD--2--Clause-green" alt="BSD-2-Clause">
</p>

![Screenshot Hero in action](./pictures/export.png)

---

<a id="english"></a>

## 🇺🇸 English

Screenshot Hero is a Linux-native screenshot annotation app built with Rust, GTK4, and Libadwaita.

The idea for this application came from a simple gap: at the time of writing, there is no screenshot and annotation app that works seamlessly with GNOME + Wayland because of Wayland's security and privacy model.

Screenshot Hero does not try to bypass Wayland directly. Instead, it integrates with GNOME's native capture tool.

Designed for an open-source workflow, it helps you move fast through:
**Capture -> Annotate -> Export/Copy**.

### Features

- Region capture through GNOME/XDG Screenshot Portal
- Open local PNG/JPEG files
- Annotation toolkit (text, shapes, arrows, blur, pixelate, redaction, and more)
- Zoom, pan, crop, undo/redo
- Export to PNG/JPEG and copy to clipboard
- Save and load `.shero` project files
- Offline-first and privacy-first: your screenshots stay on your machine

![Screenshot Hero app icon](./data/icons/hicolor/256x256/apps/com.screenshot_hero.ScreenshotHero.png)

### Quick Start

```bash
git clone https://github.com/ricrsantos/screenshot_hero.git
cd screenshot_hero
cargo run
```

Run directly in capture mode:

```bash
cargo run -- --capture
```

### Installation

#### Flathub (Recommended)

```bash
flatpak install flathub com.screenshot_hero.ScreenshotHero
flatpak run com.screenshot_hero.ScreenshotHero
```

> Coming soon: the Flathub listing is being prepared. Until then, use the local Flatpak build steps below.

#### Local Flatpak build

Manifest: `build/com.screenshot_hero.ScreenshotHero.yml`

Install required runtime/SDK:

```bash
flatpak install flathub org.gnome.Platform//50 org.gnome.Sdk//50
```

Build, install, and run:

```bash
flatpak-builder --user --install build-dir build/com.screenshot_hero.ScreenshotHero.yml --force-clean
flatpak run com.screenshot_hero.ScreenshotHero
```

Capture mode with Flatpak:

```bash
flatpak run com.screenshot_hero.ScreenshotHero --capture
```

### GNOME Shortcut Tip

You can assign Screenshot Hero capture mode to a GNOME custom keyboard shortcut:

```bash
flatpak run com.screenshot_hero.ScreenshotHero --capture
```

If you prefer, you can even replace GNOME's default screenshot shortcut and bind Screenshot Hero to `[PrintScr]`.

### Requirements (Development)

- Rust stable (via [rustup](https://rustup.rs/))
- GTK4 and Libadwaita development libraries
- GNOME/Wayland (or X11)
- XDG Desktop Portals (`org.freedesktop.portal.Desktop`)

**Fedora**

```bash
sudo dnf install gtk4-devel libadwaita-devel gdk-pixbuf2-devel gcc pkg-config
```

**Debian / Ubuntu**

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libgdk-pixbuf-2.0-dev build-essential pkg-config
```

**Arch Linux**

```bash
sudo pacman -S gtk4 libadwaita gdk-pixbuf-2.0 base-devel
```

### Build and Test

```bash
cargo build
cargo test --lib
```

Release build:

```bash
cargo build --release
```

### Contributing

Contributions are welcome.

1. Open an issue for bugs, UX feedback, or feature requests.
2. Fork the repo and create a branch from `main`.
3. Keep changes focused and include tests when possible.
4. Run:

```bash
cargo build
cargo test --lib
```

5. Open a Pull Request with a clear description and screenshots/GIFs when UI changes are involved.

### Project Structure

```text
src/
├── main.rs
├── application.rs
├── capture/
├── annotations/
├── canvas/
├── export/
├── persistence/
└── ui/
build/
└── com.screenshot_hero.ScreenshotHero.yml
```

### License

BSD 2-Clause. See [LICENSE](LICENSE).

---

<a id="portugues-br"></a>

## 🇧🇷 Português (BR)

O Screenshot Hero é um aplicativo nativo Linux para anotação de capturas de tela, desenvolvido com Rust, GTK4 e Libadwaita.

A ideia deste aplicativo surgiu de uma lacuna simples: até o momento desta escrita, não existe um app de captura e anotação de tela que funcione de forma fluida com GNOME + Wayland por causa do modelo de segurança e privacidade do Wayland.

O Screenshot Hero não tenta contornar o Wayland diretamente. Em vez disso, integra-se com a ferramenta nativa de captura do GNOME.

Projetado para um fluxo open source, ele ajuda você a avançar rapidamente em:
**Capturar -> Anotar -> Exportar/Copiar**.

### Recursos

- Captura de região via GNOME/XDG Screenshot Portal
- Abertura de arquivos locais PNG/JPEG
- Ferramentas de anotação (texto, formas, setas, blur, pixelate, redaction e mais)
- Zoom, pan, crop, desfazer/refazer
- Exportação em PNG/JPEG e cópia para a área de transferência
- Salvamento e carregamento de projetos `.shero`
- Offline e com privacidade: as imagens ficam na sua máquina

![Ícone do app Screenshot Hero](./data/icons/hicolor/256x256/apps/com.screenshot_hero.ScreenshotHero.png)

### Início Rápido

```bash
git clone https://github.com/ricrsantos/screenshot_hero.git
cd screenshot_hero
cargo run
```

Para iniciar direto no modo de captura:

```bash
cargo run -- --capture
```

### Instalação

#### Flathub (Recomendado)

```bash
flatpak install flathub com.screenshot_hero.ScreenshotHero
flatpak run com.screenshot_hero.ScreenshotHero
```

> Em breve: a publicação no Flathub está em preparação. Até lá, use os passos de build local com Flatpak abaixo.

#### Build local com Flatpak

Manifesto: `build/com.screenshot_hero.ScreenshotHero.yml`

Instale o runtime/SDK necessários:

```bash
flatpak install flathub org.gnome.Platform//50 org.gnome.Sdk//50
```

Build, instalação e execução:

```bash
flatpak-builder --user --install build-dir build/com.screenshot_hero.ScreenshotHero.yml --force-clean
flatpak run com.screenshot_hero.ScreenshotHero
```

Modo de captura com Flatpak:

```bash
flatpak run com.screenshot_hero.ScreenshotHero --capture
```

### Dica de Atalho no GNOME

Você pode atribuir o modo de captura do Screenshot Hero a um atalho de teclado personalizado no GNOME:

```bash
flatpak run com.screenshot_hero.ScreenshotHero --capture
```

Se quiser, você pode inclusive substituir o atalho padrão de captura do GNOME e vincular o Screenshot Hero à tecla `[PrintScr]`.

### Requisitos (Desenvolvimento)

- Rust estável (via [rustup](https://rustup.rs/))
- Bibliotecas de desenvolvimento GTK4 e Libadwaita
- Sessão GNOME/Wayland (ou X11)
- XDG Desktop Portals (`org.freedesktop.portal.Desktop`)

**Fedora**

```bash
sudo dnf install gtk4-devel libadwaita-devel gdk-pixbuf2-devel gcc pkg-config
```

**Debian / Ubuntu**

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libgdk-pixbuf-2.0-dev build-essential pkg-config
```

**Arch Linux**

```bash
sudo pacman -S gtk4 libadwaita gdk-pixbuf-2.0 base-devel
```

### Build e Testes

```bash
cargo build
cargo test --lib
```

Build de release:

```bash
cargo build --release
```

### Como Contribuir

Contribuições são muito bem-vindas.

1. Abra uma issue para bugs, feedback de UX ou solicitações de funcionalidade.
2. Faça fork do repositório e crie uma branch a partir da `main`.
3. Mantenha as alterações focadas e inclua testes quando possível.
4. Execute:

```bash
cargo build
cargo test --lib
```

5. Abra um Pull Request com uma descrição clara e screenshots/GIFs quando houver alterações na interface.

### Estrutura do Projeto

```text
src/
├── main.rs
├── application.rs
├── capture/
├── annotations/
├── canvas/
├── export/
├── persistence/
└── ui/
build/
└── com.screenshot_hero.ScreenshotHero.yml
```

### Licença

BSD 2-Clause. Veja [LICENSE](LICENSE).
