# Roadmap: Nive — Framework de UI para Desktop em Rust/Iced

**Nive** é um framework de **propósito geral** para aplicações desktop em **Rust/Iced**, com
foco em **Developer Experience (DX)** e **performance previsíveis** — e projetado para ser
**adequado também a aplicações de alta densidade de dados** (analíticas, de engenharia,
power-user) sem impor essa complexidade a quem não precisa dela.

O princípio organizador é: **core minimalista, capacidades pesadas opt-in**. Um app simples
(counter, formulário, CRUD) compila com o prelúdio estável e zero peso extra; um app denso
ativa features como `tables`, `charts` ou `devtools` quando precisa.

> **Status deste documento:** revisão que reconcilia o roadmap com o que já existe no
> repositório e reescopa itens para refletir a estratégia de propósito geral. Veja
> [§7 Mudanças nesta revisão](#7-mudanças-nesta-revisão).

**Legenda:** ✅ Pronto · 🟡 Parcial · ⬜ A fazer · ✂️ Reescopado/adiado

---

## 📑 Índice
1. [Visão & Filosofia](#1-visão--filosofia)
2. [Estado Atual (fundação já construída)](#2-estado-atual-fundação-já-construída)
3. [Mapa de Capacidades (checklist com status)](#3-mapa-de-capacidades-checklist-com-status)
4. [Plano de Execução (milestones)](#4-plano-de-execução-milestones)
5. [Modularidade por Features](#5-modularidade-por-features)
6. [Requisitos de Estabilidade e Publicação](#6-requisitos-de-estabilidade-e-publicação)
7. [Mudanças nesta revisão](#7-mudanças-nesta-revisão)

---

## 1. Visão & Filosofia

* **Propósito geral, com afinidade para densidade.** Nive serve apps desktop comuns com
  excelente DX e escala para UIs densas de dados — sem forçar essa complexidade em quem
  não precisa. Densidade é um *modo*, não o único caminho.
* **DX em primeiro lugar.** Prelúdios estáveis em camadas, scaffolding via CLI (`nive new`),
  contratos type-safe e boilerplate mínimo. O caminho feliz deve ser curto.
* **Performance previsível (não dogmática).** A `view` é declarativa e evita alocação/clone
  redundante nos caminhos quentes. Nota de realidade: o `Element` do Iced é
  `Box<dyn Widget>` por design — o alvo não é "zero `Box`", e sim **não alocar além do que
  o Iced já exige** e manter taxas de atualização estáveis sob carga.
* **Type-Safe UI.** Usar o sistema de tipos do Rust para forçar estados válidos de ponta a
  ponta, blindando contra inputs inválidos em runtime.
* **Densidade de informação quando desejada.** Escala de espaçamento compacta, sem paddings
  exagerados de estilo web-mobile; cada pixel pode trabalhar a favor do dado.
* **Segurança de memória.** `unsafe` restrito ao mínimo (hoje: 2 ocorrências — o FFI objc2
  do ícone de app no macOS e um `transmute_copy` da janela-unit no *program runner*).

---

## 2. Estado Atual (fundação já construída)

Resumo honesto do que já existe (crates `nive-ui`, `nive-runtime`, `nive-runtime-derive`,
`nive`, `nive-cli`). Isto **não** é trabalho a fazer — é a base sobre a qual o roadmap avança.

* **Design system:** tokens (cor, spacing compacto base-4, radius, shadow, tipografia),
  theming semântico por *roles* (`SurfaceRole`, `TextRole`, `BorderRole`, `ToneRole`),
  catálogos Light/Dark, 40+ widgets primitivos.
* **Runtime:** `Application`/`Update`/`Context`, lifecycle, bootstrap/splash, **multi-janela
  completo** (`WindowSpec`/`WindowRegistry`/handshakes), máquinas de estado assíncrono
  (`Resource`, `Operation`, `OperationRegistry`), feedback (toasts em fila com severidade,
  `UserFacingError`), settings/session persistidos (serde).
* **DX:** devtools opt-in (inspeção + simuladores via `#[derive(Inspect)]`),
  atalhos/navegação por teclado (`ShortcutMap`, `focus_trap`), command palette, CLI de
  scaffolding e de ícones, sistema de ícones Lucide tipado e empacotado em build-time.
* **Qualidade:** testes de contrato + `trybuild`, docs por crate, CI de readiness.

---

## 3. Mapa de Capacidades (checklist com status)

As fases são **áreas temáticas**, não etapas estritamente sequenciais. A ordem de execução
real está em [§4](#4-plano-de-execução-milestones).

### Fase 1 — Fundações, Layout Denso e Dados Assíncronos

- **Sistema de Design Tokens de Alta Fidelidade**
  - ✅ Paletas funcionais estritas (Muted, Border, Active/Focus, Success, Warning,
    Critical/Destructive) — `theme/roles.rs`.
  - ✅ Escala de espaçamento compacta (micro-ajustes de padding/gap) — `tokens/spacing.rs`.
  - ⬜ Integração com **OKLCH** para manipulação cromática perceptualmente uniforme e
    perfis de alto contraste. *Hoje os tokens são hex/RGB (`tokens/color.rs`); OKLCH é a
    base que destrava paletas geradas e contraste garantido.*
- **Controles de Input Técnico com Validação Estrita**
  - ✅ Slot nativo para sufixos/rótulos no container do input (unidades, tipos) —
    `InputGroup` (`leading_text`/`trailing_text`/`trailing_icon`).
  - ⬜ `NumericInputField<T>` com validação em digitação, limites Min/Max, incremento por
    Step e notação científica. *Hoje há `Input` string-based com `FieldValidation`, mas
    sem tipo numérico genérico.*
- **Ingestão Assíncrona de Dados (Streams)**
  - ✅ Request/response assíncrono com guarda de staleness — `Resource`/`Operation`.
  - ⬜ Helpers para encapsular `iced::Subscription` sobre canais contínuos
    (`tokio::sync::mpsc` / `futures::stream`), convertendo pacotes de background em
    `Message` sem bloquear a thread principal. *Esta é a peça "stream" propriamente dita,
    distinta das máquinas request/response já existentes.*

### Fase 2 — Componentes Analíticos e Visualização (opt-in)

> Capacidades pesadas, atrás de Cargo features. O core não as carrega.

- **Data Tables Densas e Virtualizadas** (`feature = "tables"`)
  - ⬜ Virtual scrolling (culling por viewport) para dezenas de milhares de registros sem
    degradação de frames. *Iced não tem virtualização nativa — é um widget bespoke.*
  - ⬜ Cabeçalhos interativos: ordenação multi-coluna e filtros por predicado.
  - ⬜ Células compactas com barras de progresso / indicadores inline (*micro-charts*).
- **Plotagem de Alta Performance** (`feature = "charts"`)
  - ⬜ Canvas para séries temporais contínuas e séries numéricas. *Avaliar integrar
    `plotters`/`plotters-iced` antes de escrever do zero.*
  - ⬜ Subamostragem (*downsampling*) em runtime conforme a resolução horizontal.
- **Ecossistema de Ícones Tipado e Eficiente** (`feature = "icons"`)
  - ✅ Roles semânticas (`IconRole`), símbolos gerados (`IconSymbol`), cor por token, escala e rotação — `Icon`.
  - ✅ Empacotamento build-time (CLI `nive icons`) embarcando só os glifos usados.
  - ✂️ Controle de espessura de linha **em runtime** — adiado. O `stroke_width` build-time
    no manifesto já cobre o caso comum; runtime tem baixo valor/custo alto.

### Fase 3 — Workspace, Multi-Janela e Estado Escalável

- **Suporte Multi-Janelas** ✅
  - ✅ API para abrir/focar/fechar janelas e handshakes de exit — `lifecycle/window.rs`.
  - ✅ Sincronização inter-janelas sob arquitetura Elm (estado central, despacho síncrono).
- **Painéis e Abas** ✅
  - ✅ Tabs, painéis colapsáveis e split panes — `tabs.rs`, `panel.rs`, `split_pane.rs`.
  - 🟡 Garantir preservação/memória de estado da visualização anterior em todos os casos.
- **Arquitetura de Estado Modular**
  - 🟡 Composição de telas via `ScreenView`/`ScreenUpdate`.
  - ⬜ Trait/macro utilitária para aninhar sub-módulos (State, Message, Update) reduzindo
    boilerplate e isolando efeitos colaterais. *A peça que falta para apps grandes.*

### Fase 4 — Resiliência, Alertas e Feedback Determinístico

- **Alertas e Notificações** ✅🟡
  - ✅ Toasts temporários em fila, classificados por severidade/tom e posição —
    `feedback/toast.rs` (`VecDeque`, `ToastTone`, `ToastPosition`).
  - 🟡 Central/log persistente de notificações com carimbo de data/hora (*timestamp*).
    *Há `RuntimeEventLog` em `support/`, ainda não conectado como histórico de alertas.*
- **Micro-interações Determinísticas** ✅
  - ✅ Transições rápidas/lineares sem delays elásticos — `widgets/animation.rs` (`Easing`).
  - ✅ Estados de carregamento explícitos — `skeleton.rs`, `ProgressBar`, `Spinner`.

### Fase 5 — DX Pro: Inspeção, Simulação e Diagnóstico

> Tudo atrás de `feature = "devtools"` / `#[cfg(debug_assertions)]`. Zero peso em release.

- **Snapshots e Simulação de Estado**
  - 🟡 Inspeção estrutural e simuladores de `Resource`/`Operation` via `#[derive(Inspect)]`
    — `inspect.rs`, `devtools/`.
  - ⬜ Serialização/desserialização serde da **árvore de State real** para JSON/TOML
    (salvar/restaurar cenário exato p/ reprodução de bug). *Hoje o snapshot é estrutural,
    não um round-trip dos valores reais.*
  - 🟡 Geradores de mock — probe simula desfechos; falta gerador de **stream de alta
    frequência** alimentando `Subscriptions`.
- **Inspetor de Layout e Overflow** ⬜
  - ⬜ Modo visual (estilo DevTools) com bordas, zonas de padding/margin e alerta de
    *overflow/clipping* sob densidade alta.
- **Captura de Mensagens / Time-Travel** ✂️ (reescopado)
  - ⬜ Log opcional de `Message` em **ring-buffer** com janela limitada.
  - ✂️ Time-travel com mutações inversas e histórico ilimitado **adiado/condicional**:
    sob streams de alta frequência (caso-alvo) reter histórico completo estoura memória.
    Só faz sentido como captura opt-in de janelas curtas.
- **Hot-Reload de Recursos** ✂️ (reescopado, baixa prioridade)
  - ⬜ File-watching apenas de **theme/tokens em JSON** em modo debug. *Tokens hoje são
    `const fn` compile-time; hot-reload exige tabelas de runtime — mudança grande, valor
    médio num fluxo Rust onde recompilar é o normal.*

### Fase 6 — Produtividade Operacional (Keyboard-First) e i18n

- **Navegação via Teclado** ✅
  - ✅ Atalhos globais customizáveis e fluxo de foco (`Tab`) — `ShortcutMap`,
    `keyboard_navigation`, `focus_trap`, command palette.
- **Internacionalização** (item dividido por valor/custo)
  - ⬜ **Formatação localizada** de números, datas e durações (micro/segundos). *Alto valor
    para apps de dados, baixo custo — fazer primeiro.*
  - ⬜ Dicionário runtime via **Project Fluent**. *Menor prioridade para ferramentas
    técnicas frequentemente mono-locale — fazer depois.*

---

## 4. Plano de Execução (milestones)

Ordem recomendada, cruzando as fases. Cada milestone entrega valor isolado.

| Milestone | Entrega | Itens |
|-----------|---------|-------|
| **M0 — Base cromática & reconciliação** | Fundação que destrava o resto | OKLCH nos tokens · reconciliar este roadmap (feito) |
| **M1 — Inputs técnicos & formatação** | Apps de engenharia usáveis | `NumericInputField<T>` (Min/Max/Step/científica) · formatação localizada de números/datas |
| **M2 — Tabela densa** | `feature = "tables"` | Widget de tabela: virtual scrolling · sorting multi-coluna · filtros por predicado · células com micro-charts |
| **M3 — Visualização** | `feature = "charts"` | Série temporal + downsampling (avaliar `plotters-iced`) |
| **M4 — Streams & estado escalável** | Tempo real + apps grandes | Helpers `mpsc`/`stream` → `Subscription` · macro/trait de estado modular · mock streams (devtools) |
| **M5 — DX Pro** | Diagnóstico sério | Snapshot serde do State real (save/load) · inspetor de layout/overflow · captura de mensagens em ring-buffer |
| **M6 — i18n pleno & polish** | Internacional + acabamento | Dicionário Fluent runtime · hot-reload de theme JSON · central de notificações com timestamp |

**Prova viva:** a cada milestone de capacidade densa (M2–M4), adicionar **um exemplo de
dashboard com carga massiva de dados simulados** em `/examples`, demonstrando a tese do
projeto sob estresse real.

---

## 5. Modularidade por Features

Core mínimo; capacidades pesadas opt-in. Estado atual e alvo:

| Feature | Status | Observação |
|---------|--------|------------|
| `devtools` | ✅ existe | Inspeção/simulação; off por padrão |
| `file-picker` | ✅ existe | `rfd`; off por padrão |
| `tables` | ⬜ alvo | Tabela virtualizada (M2) |
| `charts` | ⬜ alvo | Plotagem/downsampling (M3) |
| `i18n` | ⬜ alvo | Fluent + formatação localizada (M1/M6) |
| `icons`, `multi-window` | ✅ sempre-on | Avaliar se vale gate; hoje são leves o suficiente |

Regra: **zero payload ocioso** — um app que não usa tabelas/gráficos não paga por eles no
binário nem no tempo de compilação.

---

## 6. Requisitos de Estabilidade e Publicação

1. **Modularidade por features** — segmentar capacidades robustas em Cargo features
   independentes (ver §5). ✅ padrão estabelecido (`devtools`, `file-picker`); ⬜ estender.
2. **Segurança de memória estrita** — ✅ restrição a `unsafe` honrada (2 ocorrências: FFI do
   ícone de app + `transmute_copy` da janela-unit no program runner). Manter como gate de CI.
3. **Documentação e amostragem técnica** — 🟡 8 exemplos existem, porém pequenos. ⬜ falta a
   amostragem-âncora: dashboards com mock streams massivos, tabela virtualizada, gráficos,
   janela de diagnóstico destacável, formulário com teclado numérico, e import/export de
   snapshot de estado.
4. **Estabilidade de API** — prelúdios em camadas já estabilizam a superfície mínima;
   manter APIs feature-gated (devtools/inspect) marcadas como beta até 1.0.

---

## 7. Mudanças nesta revisão

* **Posicionamento corrigido:** de "framework *exclusivo* de alta densidade" para
  "**propósito geral, adequado também a alta densidade**, com DX e performance".
* **Checklist reconciliado com a realidade:** fases 3 e 4 já estão majoritariamente prontas;
  multi-janela, tabs/painéis, toasts, teclado, animações, ícones, settings e devtools-de-
  estado marcados como ✅/🟡 em vez de tudo `[ ]`.
* **Diretriz de performance ajustada:** "zero-cost / sem `Box` na view" reescrita para um
  alvo realista (o `Element` do Iced é boxed por design).
* **Itens reescopados:** time-travel → captura opt-in em ring-buffer; hot-reload → só theme
  JSON, baixa prioridade; espessura de ícone em runtime → adiada.
* **i18n dividido:** formatação localizada (alta prioridade) separada do dicionário Fluent
  (baixa prioridade).
* **Plano de execução por milestones** adicionado, separando "o quê" (capacidades) de
  "quando" (ordem), com exemplos-âncora de dashboard a cada milestone denso.
