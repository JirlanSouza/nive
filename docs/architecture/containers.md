# L2 — Diagrama de Contêineres (crates)

Os contêineres do Nive são os **crates** do workspace. Cada um é uma unidade compilável
publicável no crates.io.

```mermaid
flowchart LR
    dev([App Developer])

    subgraph nive["Workspace Nive"]
        direction TB
        umbrella["nive<br/>crate (umbrella)<br/>Re-exporta ui+runtime; define os prelude tiers estáveis; forwarda features devtools/file-picker"]
        ui["nive-ui<br/>crate<br/>Design system: tokens, theme semântico por roles, 40+ widgets, hosts de overlay. Depende de iced e nive-core."]
        rt["nive-runtime<br/>crate<br/>Application/Update, lifecycle, multi-janela, máquinas de estado async, feedback, settings, devtools"]
        core["nive-core<br/>crate<br/>Contratos de apresentação neutros (erro, toast, status). Zero dependências."]
        derive["nive-runtime-derive<br/>proc-macro crate<br/>#[derive(Inspect)] para travessia de estado nos devtools"]
        cli["nive-cli<br/>binary crate<br/>nive new (scaffold, com --dashboard) e nive icons (manifest provider-neutral)"]
    end

    iced["Iced 0.14<br/>Runtime GUI / wgpu"]
    rfd["rfd / objc2 / winres<br/>FFI de plataforma (feature-gated)"]
    icon_providers["Icon providers<br/>Lucide via ureq/cache e SVGs custom locais"]

    dev -->|use nive::prelude::*| umbrella
    umbrella -->|re-exporta| ui
    umbrella -->|re-exporta| rt
    rt -->|depende (re-exporta APIs estáveis)| ui
    rt -->|usa<br/>Inspect| derive
    rt -->|depende (contratos de apresentação)| core
    ui -->|depende (contratos de apresentação)| core
    ui -->|depende| iced
    rt -->|depende| iced
    rt -->|platform/ (file-picker, app_icon)| rfd
    dev -->|cargo install nive-cli| cli
    cli -->|nive icons compila refs de| icon_providers
    cli -->|gera projetos que dependem de<br/>templates| umbrella

    classDef person fill:#f7f7f7,stroke:#666,color:#222;
    classDef container fill:#e8f1ff,stroke:#4b77be,color:#111;
    classDef external fill:#eee,stroke:#999,color:#333;
    class dev person;
    class umbrella,ui,rt,core,derive,cli container;
    class iced,rfd,icon_providers external;
```

## Grafo de dependências (compilação)

Camadas estritas, sem ciclos. `nive-ui` é a camada mais baixa e **não conhece** o runtime.

```mermaid
flowchart BT
    iced["iced 0.14"]
    derive["nive-runtime-derive<br/><i>syn · quote · proc-macro2</i>"]
    core["nive-core<br/><i>zero deps</i>"]
    ui["nive-ui<br/><i>→ iced, core</i>"]
    rt["nive-runtime<br/><i>→ ui, core, derive, iced</i><br/>serde · tokio · log · rfd?"]
    umbrella["nive (umbrella)<br/><i>→ ui, rt</i>"]
    cli["nive-cli<br/><i>standalone</i><br/>clap · include_dir · toml · ureq"]

    ui --> iced
    ui --> core
    rt --> ui
    rt --> core
    rt --> derive
    rt --> iced
    umbrella --> ui
    umbrella --> rt

    classDef ext fill:#eee,stroke:#999,color:#333;
    class iced ext;
```

## Notas

| Crate | Papel | Features | Observação |
|-------|-------|----------|------------|
| `nive-core` | Contratos de apresentação | `default = []` | Camada mais baixa; **zero dependências**, nem `iced` |
| `nive-ui` | Design system | `default = []` | Camada base de UI; depende de `iced` e `nive-core` |
| `nive-runtime` | Runtime/lifecycle | `devtools`, `file-picker` | Cola tudo; re-exporta APIs estáveis de `nive-ui` e implementa os contratos de `nive-core` |
| `nive-runtime-derive` | Proc-macro | `devtools` | `#[derive(Inspect)]`; vira no-op sem a feature |
| `nive` | Umbrella + prelude | `devtools`, `file-picker` (forward) | Superfície estável recomendada para apps |
| `nive-cli` | Ferramenta de DX | — | **Não** depende dos crates do framework; gera/baixa via templates e rede |

- **`nive-cli` é desacoplado:** scaffolda projetos por templates embutidos (`include_dir`) e
  compila `icons.toml` em módulos/assets checados. Não linka contra
  `nive`/`nive-ui`/`nive-runtime`.
- O flag **`nive new --dashboard`** já existe na CLI — gera uma variante de app voltada a
  dashboard (ponto de partida natural para os exemplos densos do roadmap).
