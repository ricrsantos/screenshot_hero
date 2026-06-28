# PoC 02 – Captura de Screenshot no GNOME Wayland utilizando Rust + XDG Desktop Portal

## Objetivo

Validar a viabilidade de implementar a funcionalidade **"Nova Captura"** do projeto Screenshot Hero em ambientes GNOME/Wayland utilizando mecanismos oficialmente suportados pelo sistema operacional.

O objetivo desta PoC foi comprovar que uma aplicação Rust consegue:

1. Abrir a interface nativa de captura de tela do GNOME.
2. Permitir que o usuário selecione uma área da tela.
3. Receber o arquivo gerado.
4. Obter programaticamente a URI da imagem capturada.

---

# Ambiente

* Sistema Operacional: Linux
* Desktop Environment: GNOME
* Display Server: Wayland
* Linguagem: Rust
* Biblioteca: ashpd
* Portal: XDG Desktop Portal

---

# Investigação Inicial

## Tentativa utilizando API privada do GNOME Shell

Comando executado:

```bash
gdbus call \
  --session \
  --dest org.gnome.Shell \
  --object-path /org/gnome/Shell/Screenshot \
  --method org.gnome.Shell.Screenshot.InteractiveScreenshot
```

### Resultado

```text
Error: GDBus.Error:org.freedesktop.DBus.Error.AccessDenied: InteractiveScreenshot is not allowed
```

### Análise

A interface existe, porém o GNOME restringe o acesso para aplicações externas.

### Conclusão

Não é viável utilizar:

```text
org.gnome.Shell.Screenshot.InteractiveScreenshot
```

para implementar o Screenshot Hero.

---

# Verificação do XDG Desktop Portal

Comando executado:

```bash
gdbus introspect \
  --session \
  --dest org.freedesktop.portal.Desktop \
  --object-path /org/freedesktop/portal/desktop
```

### Resultado

Foi identificada a interface:

```text
org.freedesktop.portal.Screenshot
```

Versão:

```text
version = 2
```

Também foi identificada:

```text
org.freedesktop.portal.ScreenCast
```

Versão:

```text
version = 5
```

### Análise

O sistema possui suporte completo aos portais necessários para captura de tela.

---

# Primeira chamada ao portal

Comando executado:

```bash
gdbus call \
  --session \
  --dest org.freedesktop.portal.Desktop \
  --object-path /org/freedesktop/portal/desktop \
  --method org.freedesktop.portal.Screenshot.Screenshot \
  "" \
  "{}"
```

### Resultado

```text
(objectpath '/org/freedesktop/portal/desktop/request/1_252/t',)
```

Nenhuma interface gráfica foi exibida.

### Análise

A chamada foi aceita.

Entretanto, sem parâmetros adicionais, o portal não abriu a interface de captura.

---

# Captura interativa

Comando executado:

```bash
gdbus call \
  --session \
  --dest org.freedesktop.portal.Desktop \
  --object-path /org/freedesktop/portal/desktop \
  --method org.freedesktop.portal.Screenshot.Screenshot \
  "" \
  "{'interactive': <true>}"
```

### Resultado

O GNOME abriu sua interface nativa de captura de tela.

O usuário conseguiu:

* selecionar uma área;
* concluir a captura;
* salvar a imagem.

### Análise

A opção:

```text
interactive=true
```

é necessária para abrir a interface gráfica de captura.

### Conclusão

O portal atende perfeitamente ao requisito de seleção de região da tela.

---

# Criação da PoC em Rust

## Criação do projeto

```bash
cargo new screenshot-poc
cd screenshot-poc
```

---

## Dependências

Arquivo `Cargo.toml`:

```toml
[dependencies]
ashpd = "0.12"
tokio = { version = "1", features = ["full"] }
```

---

## Compilação

```bash
cargo build
```

### Resultado

Compilação concluída com sucesso.

---

# Primeira implementação

Código utilizado:

```rust
use ashpd::desktop::screenshot::Screenshot;
use ashpd::WindowIdentifier;

#[tokio::main]
async fn main() -> ashpd::Result<()> {
    println!("Solicitando screenshot...");

    let response = Screenshot::request()
        .interactive(true)
        .send()
        .await?;

    println!("Resposta recebida:");
    println!("{:#?}", response);

    Ok(())
}
```

### Resultado

O GNOME abriu a interface de screenshot.

A saída foi:

```text
Request(
    "/org/freedesktop/portal/desktop/request/1_154/ashpd_r9v1kv01DC",
)
```

### Análise

O método retornou apenas o objeto de requisição.

A aplicação não aguardou a resposta final do portal.

---

# Consulta da documentação da ashpd

Foi consultada a documentação da versão instalada:

```text
ashpd v0.12.3
```

Exemplo encontrado:

```rust
let response = Screenshot::request()
    .interactive(true)
    .modal(true)
    .send()
    .await?
    .response()?;

println!("URI: {}", response.uri());
```

Foi identificado que a chamada:

```rust
.response()?
```

é necessária para aguardar o término da operação e recuperar o resultado final.

---

# Implementação final

Código utilizado:

```rust
use ashpd::desktop::screenshot::Screenshot;

#[tokio::main]
async fn main() -> ashpd::Result<()> {
    println!("Solicitando screenshot...");

    let response = Screenshot::request()
        .interactive(true)
        .modal(true)
        .send()
        .await?
        .response()?;

    println!("URI: {}", response.uri());

    Ok(())
}
```

---

# Resultado Final

Execução:

```bash
cargo run
```

Saída:

```text
Solicitando screenshot...
URI: file:///home/ricardo/Pictures/Screenshots/Screenshot%20From%202026-06-01%2023-23-13.png
```

### Fluxo validado

```text
Aplicação Rust
        ↓
ashpd
        ↓
XDG Desktop Portal
        ↓
GNOME Screenshot UI
        ↓
Seleção da região
        ↓
Captura da imagem
        ↓
Retorno da URI
        ↓
Aplicação recebe o arquivo
```

---

# Conclusões

## Resultado da PoC

A PoC foi concluída com sucesso.

Foi comprovado que uma aplicação Rust consegue:

* abrir a interface nativa de screenshot do GNOME;
* permitir seleção de região;
* funcionar corretamente em Wayland;
* utilizar apenas APIs oficialmente suportadas;
* receber a URI da imagem capturada.

---

## Tecnologias aprovadas

### Portal

```text
org.freedesktop.portal.Screenshot
```

### Biblioteca Rust

```text
ashpd
```

### Runtime assíncrono

```text
tokio
```

---

## Tecnologias descartadas

### GNOME Shell Screenshot API

```text
org.gnome.Shell.Screenshot.InteractiveScreenshot
```

Motivo:

```text
AccessDenied
```

---

# Impacto para o Screenshot Hero

O principal risco técnico relacionado à funcionalidade de captura foi eliminado.

A funcionalidade "Nova Captura" pode ser implementada através do fluxo:

```text
Nova Captura
        ↓
Portal Screenshot
        ↓
GNOME Screenshot UI
        ↓
Retorno da URI
        ↓
Carregamento da imagem
        ↓
Tela de anotação do Screenshot Hero
```

Não será necessário:

* utilizar APIs privadas do GNOME;
* simular atalhos de teclado;
* depender de X11;
* utilizar hacks específicos do ambiente gráfico.

---

# Próxima PoC Recomendada

Validar se a técnica funciona para aplicações rodando dentro do container do flatpak
