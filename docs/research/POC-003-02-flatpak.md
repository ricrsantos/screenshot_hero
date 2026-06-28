# POC-003-02 – Validação de Captura de Screenshot via XDG Portal em Aplicação Flatpak

## Objetivo

Validar se uma aplicação Rust executando dentro de um sandbox Flatpak consegue:

1. Solicitar uma captura de tela através do XDG Desktop Portal.
2. Exibir a interface nativa de captura do GNOME.
3. Receber a URI do arquivo gerado após a captura.
4. Confirmar a integração entre:

   * Rust
   * Flatpak
   * XDG Desktop Portal
   * GNOME Screenshot
   * Wayland

---

# Contexto

Na POC-002 foi validado que uma aplicação Rust utilizando a biblioteca `ashpd` consegue solicitar screenshots através do XDG Portal quando executada diretamente no sistema operacional.

A POC-003-01 validou a configuração do ambiente Flatpak e a construção de uma imagem Flatpak contendo o binário da aplicação.

O objetivo desta etapa foi executar efetivamente a aplicação dentro do sandbox Flatpak.

---

# Estratégia Utilizada

Inicialmente foi tentado compilar a aplicação Rust dentro do próprio ambiente Flatpak.

Durante a investigação foi identificado que:

* O SDK Freedesktop não inclui Rust por padrão.
* Foi necessário instalar a extensão Rust.
* O Cargo passou a funcionar corretamente.
* O build passou a falhar apenas na obtenção de dependências do crates.io.

Como o objetivo da POC era validar a funcionalidade de screenshot e não o pipeline de empacotamento Rust, foi adotada uma abordagem simplificada:

```text
Compilar localmente
        ↓
Copiar binário para o Flatpak
        ↓
Executar no sandbox
```

---

# Binário Utilizado

## Verificação

### Comando

```bash
ls -lh target/release/screenshot-poc
```

### Resultado

```text
-rwxr-xr-x. 2 ricardo ricardo 4,6M jun  2 18:17 target/release/screenshot-poc
```

## Conclusão

O binário release foi gerado com sucesso e estava disponível para empacotamento.

---

# Manifesto Flatpak Utilizado

Arquivo:

```text
io.github.screenshothero.Poc003.yml
```

Conteúdo:

```yaml
app-id: io.github.screenshothero.Poc003

runtime: org.freedesktop.Platform
runtime-version: "25.08"
sdk: org.freedesktop.Sdk

command: screenshot-poc

finish-args:
  - --socket=wayland
  - --socket=fallback-x11
  - --share=ipc

modules:
  - name: screenshot-poc

    buildsystem: simple

    build-commands:
      - install -Dm755 screenshot-poc /app/bin/screenshot-poc

    sources:
      - type: file
        path: target/release/screenshot-poc
```

---

# Build da Imagem Flatpak

## Comando

```bash
flatpak-builder \
  --force-clean \
  build-dir \
  io.github.screenshothero.Poc003.yml
```

## Resultado

Trecho final da saída:

```text
Committing stage build-screenshot-poc to cache
Cleaning up
Committing stage cleanup to cache
Finishing app
Please review the exported files and the metadata
Committing stage finish to cache
Pruning cache
```

## Conclusão

O build foi concluído com sucesso.

A imagem Flatpak foi gerada corretamente.

---

# Execução da Aplicação no Sandbox

## Comando

```bash
flatpak-builder \
  --run \
  build-dir \
  io.github.screenshothero.Poc003.yml \
  screenshot-poc
```

---

# Comportamento Observado

## Passo 1

A aplicação iniciou normalmente.

Saída:

```text
Solicitando screenshot...
```

---

## Passo 2

A interface nativa de screenshot do GNOME foi exibida.

Foi possível:

* selecionar uma área;
* cancelar a operação;
* concluir a captura.

O comportamento foi idêntico ao observado fora do Flatpak.

---

## Passo 3

Após a conclusão da captura, a aplicação recebeu a URI retornada pelo Portal.

Saída:

```text
URI: file:///run/user/1000/doc/b5d93d61/Screenshot%20From%202026-06-02%2018-34-42.png
```

---

# Comparação com a POC-002

## Fora do Flatpak

Retorno típico:

```text
file:///home/ricardo/Pictures/Screenshots/...
```

---

## Dentro do Flatpak

Retorno observado:

```text
file:///run/user/1000/doc/b5d93d61/Screenshot%20From%202026-06-02%2018-34-42.png
```

---

# Análise Técnica

O caminho retornado dentro do Flatpak é significativamente diferente.

Em vez de fornecer acesso direto ao diretório:

```text
~/Pictures/Screenshots
```

o Portal disponibiliza o arquivo através do Documents Portal:

```text
/run/user/1000/doc/
```

Fluxo identificado:

```text
GNOME Screenshot
        ↓
XDG Screenshot Portal
        ↓
Documents Portal
        ↓
Aplicação Flatpak
```

Esse comportamento é esperado e faz parte do modelo de segurança do Flatpak.

O sandbox não recebe acesso direto ao sistema de arquivos do usuário.

Em vez disso, recebe acesso apenas ao arquivo autorizado pelo Portal.

---

# Descobertas

## Descoberta 1

A integração entre Rust e Flatpak funciona corretamente.

Status:

```text
VALIDADO
```

---

## Descoberta 2

A biblioteca `ashpd` funciona normalmente dentro do sandbox.

Status:

```text
VALIDADO
```

---

## Descoberta 3

O Screenshot Portal funciona normalmente dentro do Flatpak.

Status:

```text
VALIDADO
```

---

## Descoberta 4

A interface nativa de screenshot do GNOME é exibida corretamente.

Status:

```text
VALIDADO
```

---

## Descoberta 5

A aplicação recebe uma URI válida após a captura.

Status:

```text
VALIDADO
```

---

## Descoberta 6

Não foi necessário conceder acesso ao diretório Pictures.

Status:

```text
VALIDADO
```

---

## Descoberta 7

Não foi necessário utilizar permissões amplas de filesystem.

Status:

```text
VALIDADO
```

---

# Limitações da POC

Esta etapa não validou:

* abertura do arquivo retornado;
* leitura dos bytes da imagem;
* carregamento da imagem em memória;
* exibição em interface gráfica;
* anotações sobre a imagem.

---

# Estado Atual

| Item                         | Status         |
| ---------------------------- | -------------- |
| Build Flatpak                | ✅              |
| Execução no sandbox          | ✅              |
| Integração Wayland           | ✅              |
| Integração XDG Portal        | ✅              |
| Integração ashpd             | ✅              |
| Exibição da UI de screenshot | ✅              |
| Retorno da URI               | ✅              |
| Acesso ao arquivo retornado  | ⏳ Não validado |
| Exibição da imagem em UI     | ⏳ Não validado |
| Sistema de anotações         | ⏳ Não validado |

---

# Conclusão

A POC-003-02 demonstrou com sucesso que uma aplicação Rust executando dentro de um sandbox Flatpak consegue utilizar o XDG Screenshot Portal para solicitar capturas de tela.

A interface nativa do GNOME foi exibida corretamente, o usuário conseguiu concluir a captura e a aplicação recebeu uma URI válida para o arquivo gerado.

O retorno da URI através do Documents Portal demonstra que a integração está ocorrendo de acordo com o modelo de segurança esperado do Flatpak, sem necessidade de acesso direto ao sistema de arquivos do usuário.

O principal risco técnico relacionado à captura de screenshots em aplicações Flatpak pode ser considerado mitigado.

A próxima etapa recomendada é a POC-004, cujo objetivo será validar a abertura e leitura do arquivo retornado pelo Documents Portal.
