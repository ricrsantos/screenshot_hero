# Behavior Settings Extend - Specification

**Status:** Implemented  
**Feature slug:** `behavior-settings-extend`

---

## Problem Statement

O Screenshot Hero já possui configurações de automação, mas ainda não oferece controle fino sobre:

- fluxo pós-captura quando executado com `--capture`;
- tempo limitado para suprimir edição;
- encerramento automático após consumo do clipboard;
- política de abrir nova janela vs reaproveitar janela atual em novas capturas.

Isso afeta produtividade para fluxos rápidos de "capturar e colar" e gera abertura excessiva de janelas.

---

## Goals

- [x] Permitir desabilitar edição pós-captura em modo `--capture` (default desabilitado).
- [x] Permitir desabilitar edição pós-captura por período configurável (minutos/segundos; default 1 minuto, opção desabilitada).
- [ ] Encerrar automaticamente o app após a imagem sair do clipboard quando em sessão `--capture` com editor aberto (adiado).
- [x] Permitir escolher entre reaproveitar janela atual ou abrir nova janela a cada captura (default reaproveitar).

---

## User Stories

### P1: Disable Post-Capture Editing

**Como** usuário em fluxo rápido de captura  
**Quero** capturar via `--capture` sem abrir editor  
**Para** reduzir cliques e sair imediatamente do app.

**Acceptance Criteria**

1. WHEN `post-capture-editing-disabled = true` AND app starts with `--capture` THEN system SHALL invoke portal capture and quit without opening editor window.
2. WHEN `post-capture-editing-disabled = false` THEN system SHALL preserve normal `--capture` behavior.
3. Default SHALL be `false`.

**Requirement ID:** BSE-01

---

### P1: Temporary Disable Post-Capture Editing

**Como** usuário em sessão temporária de foco  
**Quero** suprimir edição por um tempo definido  
**Para** que o app se comporte como captura rápida apenas durante esse intervalo.

**Acceptance Criteria**

1. Default SHALL be disabled.
2. WHEN enabled THEN user SHALL configure minutes/seconds (default 1m 0s).
3. WHILE within duration THEN behavior SHALL be equivalent to BSE-01 enabled.
4. WHEN duration expires THEN system SHALL auto-disable temporary option and restore normal behavior.

**Requirement IDs:** BSE-02, BSE-03

---

### P1: Exit After Paste (Deferred)

**Status:** adiado para iteração futura devido inconsistências de UX entre ambientes.

### P1: Open New Window on Capture

**Como** usuário que compara capturas  
**Quero** poder abrir nova janela em cada captura  
**Para** manter imagens anteriores em paralelo.

**Acceptance Criteria**

1. Default SHALL be disabled.
2. WHEN disabled AND app window is open THEN new capture SHALL replace current image in same window.
3. WHEN enabled THEN each new capture SHALL open a new window (current legacy behavior).

**Requirement ID:** BSE-05

---

## Requirement Traceability

| Requirement ID | Description | Status |
|---|---|---|
| BSE-01 | Disable post-capture editing toggle (`--capture`) | Implemented |
| BSE-02 | Temporary disable toggle + duration settings | Implemented |
| BSE-03 | Auto-expire temporary mode and restore normal behavior | Implemented |
| BSE-04 | Exit after paste in `--capture` edit session | Deferred |
| BSE-05 | Open new window on capture toggle | Implemented |
