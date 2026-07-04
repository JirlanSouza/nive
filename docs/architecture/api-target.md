# API Alvo - Fase 1

Este documento registra o contrato publico alvo para apps Nive antes da primeira
publicacao. Ele complementa [`api-surface.md`](api-surface.md), que descreve a
superficie existente; aqui ficam as decisoes que devem orientar breaking changes,
templates, exemplos e proximas fases do roadmap.

## Principio

Um app desktop real deve conseguir começar por `nive::prelude::*`, crescer para
`nive::prelude::ui::*` quando usar estado async, dialogs ou janelas em runtime, e
recorrer a `nive::runtime::*` ou `nive::ui::*` apenas quando estiver trabalhando
diretamente na camada correspondente.

Crate-root reexports continuam convenientes antes da publicacao, mas o scaffold,
exemplos e docs devem tratar os preludes como o contrato principal.

## Prelude alvo

| Caminho | Papel | Deve conter |
|---------|-------|-------------|
| `nive::prelude::*` | Tier padrao de app e scaffold simples | `Application`, `ApplicationConfig`, `run`, `AppUpdate`, `Update`, `Context`, `ScreenView`, `Task`, `Subscription`, theme basico, `Toast`, `Action`, `ActionId`, `ActionMap`, `ShortcutMap`, `CoreEvent`, `RuntimeCommand`, tipos de declaracao de janela (`WindowSpec`, `WindowRole`, `WindowCardinality`, `WindowCommand`), settings/session basicos, erros de runtime, geometria Iced e `nive_ui::prelude::*`. |
| `nive::prelude::ui::*` | Tier estendido de app | Tudo do tier padrao mais `Resource`, `Operation`, `OperationRegistry`, `DialogRequest`, `DialogDismiss`, `ScreenUpdate`, `UserFacingError`, `BootstrapSpec`, `BrandContent`, `ToastDuration`, `ToastTone`, `WindowHandle`, `WindowRegistry`, `WindowMode`, `WindowChrome` e params de file picker quando a feature estiver ativa. |
| `nive::runtime::prelude::*` | Consumidor direto de runtime | Mesmo recorte de runtime dos tiers acima, sem depender da facade umbrella. Deve continuar util para crates que nao querem importar widgets. |
| `nive::ui::prelude::*` | Consumidor direto do design system | `Element`, `Renderer`, layout Iced comum, theme, hosts, contratos de apresentacao e widgets publicos da facade de UI. |

Decisao: manter `nive::prelude::*` e `nive::prelude::ui::*` como caminho feliz
do usuario final. `nive::runtime::prelude::*` e `nive::ui::prelude::*` sao
estaveis para consumidores por camada, mas nao devem ser necessarios no scaffold
gerado pelo CLI.

## Action, Command e Shortcut

Decisao: `Action` continua sendo o nome central para uma intencao de produto
exibivel e ativavel pelo usuario.

- `ActionId` identifica uma acao de produto de forma estavel.
- `Action<M>` carrega label, descricao, enabled state, atalho opcional e a
  mensagem emitida quando ativada.
- `ActionMap<M>` e o catalogo ordenado que alimenta atalhos, command palette,
  menus e toolbars.
- `ShortcutMap<M>` continua existindo para atalhos que nao precisam estar no
  catalogo visual de acoes.

`Command` nao substitui `Action` agora. No vocabulario alvo, `Command` fica
reservado para efeitos imperativos de runtime ou de plataforma ja existentes,
como `RuntimeCommand` e `WindowCommand`. Um `CommandRegistry` nao deve ser criado
antes do primeiro app real provar que `ActionMap` e insuficiente para menus,
toolbars, atalhos e command palette.

## RuntimeCommand e Update

Decisao: manter os nomes `Update`, `AppUpdate` e `RuntimeCommand`.

`Update<M, O, K>` e o envelope de efeitos de uma transicao: combina `Task<M>`,
um outcome opcional e uma lista ordenada de comandos de runtime. `AppUpdate<M,
K>` permanece o alias para hooks de `Application`, onde o outcome e
`Never`.

`RuntimeCommand<K>` representa apenas efeitos que o runtime executa depois do
`update` do app:

- `Toast(Toast)`
- `Window(WindowCommand<K>)`
- `Theme(ThemePreference)`
- `Exit`

Nao renomear `RuntimeCommand` para `Action` ou `Command`. A separacao alvo e:
`Action` descreve algo que o usuario pode ativar; `RuntimeCommand` descreve um
efeito que o runtime deve drenar.

## Context

Decisao: `Context` permanece pequeno, barato e somente leitura.

Ele deve expor identidade do app, tema ativo/preferido, consulta de janelas e
estado de saida. Nao deve virar service locator. Clientes de produto, repositorios
e servicos externos entram pelo `Bootstrap` e vivem no estado do app.

Novas capacidades de runtime so devem entrar em `Context` quando forem:

- globais ao runtime;
- somente leitura ou representadas por comando explicito em `Update`;
- necessarias em mais de uma area do contrato de app.

## Resource e Operation

Decisao: manter `Resource`, `Operation`, `OperationRegistry`, `RequestId` e
`Settled` como vocabulario alvo para async state.

- `Resource<T>` representa valor carregado assincronamente com
  stale-while-revalidate e rejeicao de resposta obsoleta.
- `Operation<C>` representa mutacao async sem valor persistente, preservando o
  input enquanto roda ou falha.
- `OperationRegistry` representa painel/registro de operacoes nomeadas em voo,
  especialmente para status global, progresso e cancelamento.
- `RequestId` e `Settled<T>` continuam publicos porque sao parte do contrato de
  recebimento das tasks, mas pertencem ao tier estendido.

Nao criar outra familia de nomes como `AsyncResource`, `AsyncTask` ou
`Mutation` antes do primeiro app real. A proxima evolucao deve melhorar helpers,
progresso e cancelamento sem trocar o vocabulario central.

## Toast, Error e Feedback

Decisao: manter separacao headless/runtime versus apresentacao/UI.

- `Toast` e evento user-facing temporario emitido por `Update::toast`.
- `ToastState` e fila/estado de runtime; pode continuar publico para testes,
  hosts customizados e integracao avancada, mas nao entra no caminho feliz do
  scaffold.
- `UserFacingError` e o erro apresentavel por runtime, `Resource` e `Operation`.
- Widgets de feedback em `nive-ui` devem depender de traits de apresentacao, nao
  de tipos concretos de `nive-runtime`.

Fase 3: `ToastRequest` foi removido da API publica. `Toast` e o nome alvo.

## Window lifecycle

Decisao: manter o modelo atual de janelas como contrato alvo.

- `WindowSpec` declara aparencia e comportamento inicial.
- `WindowRole` separa janelas de app e auxiliares.
- `WindowCardinality` limita single versus multiple.
- `WindowCommand<K>` e o efeito emitido pelo app para abrir, focar ou fechar
  janelas.
- `WindowHandle<K>` e `WindowRegistry<K>` representam estado de runtime e ficam
  no tier estendido.
- `WindowMode` e `WindowChrome` permanecem conceitos publicos, mas nao precisam
  estar no tier minimo.

Fase 3: `open_window` virou helper interno do runtime/devtools. Apps devem emitir
`WindowCommand` via `Update`. `WindowRegistration` permanece fora dos preludes e
fica acessivel pelo modulo `nive_runtime::application` apenas como tipo retornado
pelos introspectores de `ApplicationConfig`.

## Decisao sobre nive-core

Decisao original da Fase 1: nao criar `nive-core` naquele momento.

Motivo: os candidatos atuais ainda pertencem claramente a uma camada existente.
`Error`/`Result` sao entrada do runtime; `RequestId`, `OperationId` e `ActionId`
tem semantica ligada a runtime/estado/acoes; metadata e capabilities ainda nao
formam um contrato compartilhado independente de UI e runtime.

Decisao revisada apos a Fase 3: criar um `nive-core` minimo na Fase 4 para
contratos compartilhados de apresentacao/status.

Motivo: `nive-ui` definia traits como `ErrorPresentation`,
`ResourceStatusPresentation`, `OperationStatusPresentation` e
`ToastPresentation`, enquanto `nive-runtime` implementava esses traits para
`UserFacingError`, `Resource<T>`, `Operation<C>` e `ToastItem`. Isso criava uma
fronteira conceitual invertida: a UI definia contratos headless que descrevem
estado e feedback do runtime.

**Executado na Fase 4:** `nive-core` existe como membro do workspace
(`crates/nive-core`, zero dependencias). Os quatro traits e `ToastTone`
migraram para la; `nive-ui` reexporta-os nos mesmos caminhos publicos
(`widgets`, `widgets::feedback`, `overlays`, `prelude`, raiz para
`ToastPresentation`/`ToastTone`) e `nive-runtime` os implementa importando de
`nive_core` em vez de `nive_ui::widgets`. `nive_runtime::ToastTone` e
`nive_runtime::ToastPosition` deixaram de ser tipos proprios: o primeiro e um
reexport de `nive_core::ToastTone`, o segundo de `nive_ui::ToastPosition` —
eliminando os dois pares `impl From` duplicados e o workaround de glob
ambiguo em `crates/nive/src/lib.rs`.

Escopo do `nive-core` na Fase 4:

- contratos de apresentacao de erro (`ErrorPresentation`);
- contratos de apresentacao de toast (`ToastPresentation`);
- contratos de status de resource/operation (`ResourceStatusPresentation`,
  `OperationStatusPresentation`);
- `ToastTone`, movido porque `ToastPresentation::tone()` o retorna e a
  duplicacao runtime/UI ja tinha vazado para o usuario.

Fora do escopo inicial (permanece nas camadas atuais):

- `Resource`, `Operation`, `OperationRegistry`;
- `UserFacingError`, `Toast`, `ToastState`, `ToastItem`;
- `Action`, `ActionMap`;
- `WindowSpec`, `RuntimeCommand`;
- `Error`/`Result` de runtime;
- IDs fortes ainda ligados a uma camada especifica;
- `ToastPosition` (vocabulario de layout de UI, permanece em `nive-ui` e e
  reexportado por `nive-runtime`);
- metadata, capabilities e version sem consumidor concreto.

## Renomeacoes alvo

| Atual | Decisao |
|-------|---------|
| `ToastRequest` | Removido na Fase 3; usar `Toast`. |
| `Action` | Manter; nao renomear para `Command`. |
| `RuntimeCommand` | Manter; nao colidir com `Action`. |
| `WindowCommand` | Manter como efeito especifico de janela. |
| `Resource` / `Operation` | Manter como nomes finais de async state. |

## Remocoes ou restricoes alvo

- Nao adicionar alias `Command` para `Action`.
- Nao adicionar `CommandRegistry` ate validacao por app real.
- Nao promover `WindowRegistration` para prelude.
- Manter `open_window` como helper interno; apps usam `WindowCommand`.
- `ToastRequest` ja foi removido; nao reintroduzir alias legado.
- Tratar crate-root wildcard usage como conveniencia beta; templates e exemplos
  devem preferir os preludes.

## Criterio de aceite da Fase 1

- Um app novo tem caminho previsivel: `nive::prelude::*` primeiro,
  `nive::prelude::ui::*` quando precisar do tier estendido.
- `Action`, `RuntimeCommand` e `WindowCommand` nao competem semanticamente.
- `Context` nao acumula servicos de produto.
- `Resource` e `Operation` sao o vocabulario final para request/response e
  mutacoes async.
- A decisao revisada sobre `nive-core` esta explicita: criar somente o core
  minimo de contratos neutros na Fase 4.
- A proxima fase pode reorganizar `nive-ui` sem rediscutir o contrato principal
  de app.
