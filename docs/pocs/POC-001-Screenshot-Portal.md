# PoC – Captura de Screenshot no GNOME Wayland para o Projeto Screenshot Hero

## Objetivo

Validar a viabilidade de iniciar uma captura de tela interativa em ambientes GNOME/Wayland para utilização no projeto Screenshot Hero, utilizando mecanismos suportados oficialmente pelo sistema operacional.

---

# Ambiente

* Sistema operacional: Linux
* Ambiente gráfico: GNOME
* Sessão: Wayland
* Linguagem alvo da aplicação: Rust

---

# Primeira tentativa: API privada do GNOME Shell

Foi testada a chamada direta ao serviço D-Bus do GNOME Shell:

```bash
gdbus call \
  --session \
  --dest org.gnome.Shell \
  --object-path /org/gnome/Shell/Screenshot \
  --method org.gnome.Shell.Screenshot.InteractiveScreenshot
```

## Resultado

Retorno:

```text
Error: GDBus.Error:org.freedesktop.DBus.Error.AccessDenied: InteractiveScreenshot is not allowed
```

## Análise

A interface existe, porém o GNOME Shell restringe o acesso a esse método para aplicações externas.

Conclusão:

* Não é possível utilizar `org.gnome.Shell.Screenshot.InteractiveScreenshot`.
* Esta abordagem não é adequada para aplicações de terceiros.

---

# Investigação do Portal XDG

Foi verificado se o portal de screenshot estava disponível no sistema.

Comando:

```bash
gdbus introspect \
  --session \
  --dest org.freedesktop.portal.Desktop \
  --object-path /org/freedesktop/portal/desktop
```

## Resultado

Foi encontrada a interface:

```text
org.freedesktop.portal.Screenshot
```

com:

```text
version = 2
```

Também foi identificada a interface:

```text
org.freedesktop.portal.ScreenCast
```

com:

```text
version = 5
```

## Análise

O sistema possui suporte completo aos portais XDG necessários para captura de tela e compartilhamento de tela.

---

# Primeira chamada ao portal Screenshot

Foi executado:

```bash
gdbus call \
  --session \
  --dest org.freedesktop.portal.Desktop \
  --object-path /org/freedesktop/portal/desktop \
  --method org.freedesktop.portal.Screenshot.Screenshot \
  "" \
  "{}"
```

## Resultado

Retorno:

```text
(objectpath '/org/freedesktop/portal/desktop/request/1_252/t',)
```

Nenhuma interface gráfica foi exibida.

## Análise

A chamada foi aceita e retornou um *request handle*.

Entretanto, a ausência da opção `interactive` fez com que nenhuma interface de captura fosse aberta.

---

# Teste com captura interativa

Foi executado:

```bash
gdbus call \
  --session \
  --dest org.freedesktop.portal.Desktop \
  --object-path /org/freedesktop/portal/desktop \
  --method org.freedesktop.portal.Screenshot.Screenshot \
  "" \
  "{'interactive': <true>}"
```

## Resultado

O GNOME abriu sua interface nativa de captura de tela.

O usuário conseguiu:

* selecionar uma área da tela;
* concluir a captura;
* salvar a imagem.

O comando retornou:

```text
(objectpath '/org/freedesktop/portal/desktop/request/1_263/t',)
```

A captura foi salva em:

```text
/home/ricardo/Pictures/Screenshots
```

## Análise

A opção:

```text
interactive=true
```

é obrigatória para abrir a interface gráfica de captura do GNOME.

O portal funciona corretamente em Wayland.

---

# Investigação do modelo assíncrono

Foi observado que o método:

```text
org.freedesktop.portal.Screenshot.Screenshot()
```

não retorna diretamente o caminho da imagem.

Em vez disso, retorna um objeto de requisição:

```text
/objectpath '/org/freedesktop/portal/desktop/request/...'
```

## Análise

O protocolo segue o modelo assíncrono dos portais XDG:

1. Aplicação solicita a captura.
2. Portal cria uma requisição.
3. Usuário interage com a interface do GNOME.
4. Portal emite uma resposta assíncrona.
5. Aplicação recebe a URI do arquivo gerado.

Portanto, uma implementação completa precisa lidar com os sinais de resposta do portal.

---

# Conclusões

## Resultado da PoC

A PoC foi considerada bem-sucedida.

Foi comprovado que:

* GNOME Wayland suporta captura interativa via portal XDG.
* Não é necessário utilizar APIs privadas do GNOME Shell.
* Não é necessário simular teclas ou eventos de teclado.
* Não é necessário depender de X11.
* O mecanismo funciona utilizando apenas interfaces oficialmente suportadas.

## Arquitetura recomendada para o Screenshot Hero

Utilizar:

```text
org.freedesktop.portal.Screenshot
```

ou, em Rust:

```text
crate ashpd
```

A crate `ashpd` abstrai:

* chamadas D-Bus;
* request handles;
* sinais assíncronos;
* serialização de parâmetros.

## Próximo passo recomendado

Criar uma PoC em Rust utilizando a crate `ashpd` para:

1. abrir a interface de captura do GNOME;
2. permitir a seleção da área;
3. receber a URI da imagem capturada;
4. exibir o caminho retornado no terminal.

Após validar esse fluxo, será possível integrar a captura ao pipeline principal do Screenshot Hero.
