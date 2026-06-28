# POC-003-01 – Validação Inicial de Execução do Screenshot Hero em Ambiente Flatpak

## Objetivo

Validar a viabilidade de executar a PoC de captura de screenshot (POC-002) dentro de um ambiente Flatpak.

O objetivo desta etapa foi verificar:

1. Disponibilidade do ambiente Flatpak.
2. Disponibilidade dos runtimes e SDKs necessários.
3. Capacidade de compilar uma aplicação Rust dentro do ambiente Flatpak.
4. Identificar possíveis bloqueios antes de validar a integração com o XDG Desktop Portal.

---

# Contexto

A POC-002 já validou com sucesso:

* Captura de screenshot via XDG Desktop Portal.
* Uso da biblioteca Rust `ashpd`.
* Execução em GNOME + Wayland.
* Retorno da URI da imagem capturada.

A POC-003 busca responder a seguinte pergunta:

> A mesma aplicação consegue funcionar quando distribuída e executada como Flatpak?

---

# Estrutura da POC

Foi criada uma nova pasta contendo uma cópia funcional da POC-002:

```text
POC-003-Flatpak/
```

O código-fonte permaneceu inalterado.

---

# Verificação do Flatpak

## Comando

```bash
flatpak --version
```

## Resultado

```text
Flatpak 1.16.6
```

## Conclusão

Flatpak instalado e operacional.

---

# Verificação dos repositórios Flatpak

## Comando

```bash
flatpak remotes
```

## Resultado

```text
Name    Options
fedora  system,oci
flathub system
```

## Conclusão

O Flathub está configurado e disponível.

---

# Verificação dos runtimes instalados

## Comando

```bash
flatpak list --runtime
```

## Resultado relevante

```text
org.freedesktop.Platform 25.08
org.gnome.Platform 49
org.gnome.Platform 50
```

## Conclusão

Os runtimes necessários estão disponíveis.

---

# Verificação dos SDKs instalados

## Comando

```bash
flatpak list --runtime | grep -i sdk
```

## Resultado

```text
org.freedesktop.Platform
```

## Análise

Não havia SDK Rust disponível inicialmente.

---

# Instalação do SDK Freedesktop

## Comando

```bash
flatpak install flathub org.freedesktop.Sdk//25.08
```

## Resultado

Instalação concluída com sucesso.

## Conclusão

O ambiente de desenvolvimento Flatpak passou a possuir um SDK compatível com o runtime utilizado.

---

# Verificação da aplicação base

## Comando

```bash
cargo run
```

## Resultado

A aplicação continuou funcionando normalmente.

Exemplo de saída:

```text
Solicitando screenshot...
URI: file:///home/ricardo/Pictures/Screenshots/...
```

## Conclusão

A baseline da aplicação permaneceu íntegra antes do empacotamento.

---

# Criação do Manifesto Flatpak

Arquivo:

```text
io.github.screenshothero.Poc003.yml
```

Conteúdo inicial:

```yaml
app-id: io.github.screenshothero.Poc003

runtime: org.freedesktop.Platform
runtime-version: "25.08"
sdk: org.freedesktop.Sdk

command: screenshot-poc

finish-args:
  - --share=network
  - --socket=wayland
  - --socket=fallback-x11

modules:
  - name: screenshot-poc
    buildsystem: simple

    build-commands:
      - cargo build --release
      - install -Dm755 target/release/screenshot-poc /app/bin/screenshot-poc

    sources:
      - type: dir
        path: .
```

---

# Primeira tentativa de build

## Comando

```bash
flatpak-builder \
  --force-clean \
  build-dir \
  io.github.screenshothero.Poc003.yml
```

## Resultado

```text
cargo: command not found
```

## Análise

O SDK Freedesktop não inclui a toolchain Rust por padrão.

---

# Investigação da extensão Rust

## Pesquisa

```bash
flatpak search rust sdk
```

## Resultado relevante

```text
org.freedesktop.Sdk.Extension.rust-stable
1.96.0
25.08
```

---

# Instalação da extensão Rust

## Comando

```bash
flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//25.08
```

## Resultado

Instalação concluída com sucesso.

---

# Confirmação da instalação

## Comando

```bash
flatpak list | grep rust-stable
```

## Resultado

```text
Rust stable
org.freedesktop.Sdk.Extension.rust-stable
1.96.0
25.08
```

## Conclusão

A extensão Rust foi instalada corretamente.

---

# Atualização do manifesto

Foi adicionado:

```yaml
sdk-extensions:
  - org.freedesktop.Sdk.Extension.rust-stable
```

---

# Nova tentativa de build

## Resultado

Ainda ocorreu:

```text
cargo: command not found
```

## Análise

A extensão estava instalada, porém não estava presente no PATH do ambiente de build.

---

# Investigação do SDK

## Comando

```bash
flatpak run --command=sh org.freedesktop.Sdk//25.08
```

### Verificação

```bash
ls /usr/lib/sdk
```

### Resultado

```text
rust-stable
```

---

# Localização do Cargo

## Comando

```bash
find /usr/lib/sdk/rust-stable -name cargo | head
```

## Resultado

```text
/usr/lib/sdk/rust-stable/bin/cargo
```

## Conclusão

O Cargo estava presente, porém fora do PATH padrão.

---

# Ajuste do PATH

Foi adicionado ao manifesto:

```yaml
build-options:
  append-path: /usr/lib/sdk/rust-stable/bin
```

---

# Resultado após ajuste

## Build

```bash
flatpak-builder \
  --force-clean \
  build-dir \
  io.github.screenshothero.Poc003.yml
```

## Saída

```text
Running: cargo build --release
Updating crates.io index
```

Posteriormente:

```text
Could not resolve host: index.crates.io
```

---

# Descobertas Importantes

## Descoberta 1

A extensão Rust foi carregada corretamente.

Evidência:

```text
Updating crates.io index
```

Isso demonstra que:

* cargo foi encontrado;
* cargo foi executado;
* toolchain Rust está funcional dentro do Flatpak.

---

## Descoberta 2

O problema atual não está relacionado ao Rust.

O problema passou a ser:

```text
Could not resolve host: index.crates.io
```

ou seja:

* resolução DNS;
* acesso à rede;
* obtenção das dependências do crates.io.

---

## Descoberta 3

O principal risco da integração Rust + Flatpak foi eliminado.

Foi comprovado que:

* SDK funciona;
* extensão Rust funciona;
* cargo funciona;
* flatpak-builder consegue iniciar um build Rust.

---

# Estado Atual da POC

| Item                                  | Status         |
| ------------------------------------- | -------------- |
| Flatpak instalado                     | ✅              |
| Flathub configurado                   | ✅              |
| SDK Freedesktop instalado             | ✅              |
| Extensão Rust instalada               | ✅              |
| Cargo disponível no sandbox           | ✅              |
| Build Rust iniciado dentro do Flatpak | ✅              |
| Download de crates                    | ❌              |
| Execução da aplicação em Flatpak      | ⏳ Não validado |
| Screenshot Portal em Flatpak          | ⏳ Não validado |
| Retorno da URI em Flatpak             | ⏳ Não validado |

---

# Conclusão Parcial

A POC-003 demonstrou que o ambiente Flatpak é capaz de executar o processo de build de uma aplicação Rust utilizando a extensão oficial `org.freedesktop.Sdk.Extension.rust-stable`.

O bloqueio atual não está relacionado ao Screenshot Hero, ao Wayland, ao XDG Desktop Portal ou à biblioteca `ashpd`.

A investigação deve continuar focando especificamente na obtenção das dependências Rust dentro do ambiente de build Flatpak, para que seja possível concluir a validação funcional da aplicação executando dentro do sandbox.
