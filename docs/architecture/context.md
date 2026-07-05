# L1 — Diagrama de Contexto

Nive como caixa-preta entre as pessoas que o usam e os sistemas externos sobre os quais se
apoia.

```mermaid
flowchart LR
    dev([App Developer<br/>Constrói aplicações desktop em Rust/Iced usando o Nive])
    user([Usuário Final<br/>Usa o app desktop construído com Nive])

    nive["Nive Framework<br/>Framework Rust/Iced de propósito geral, adequado também a UIs de alta densidade de dados.<br/>Design system + runtime + DX tooling."]

    iced["Iced 0.14<br/>Runtime GUI com arquitetura Elm, renderização via wgpu, canvas e svg"]
    os["Plataforma do SO<br/>macOS, Windows, Linux: janelas, input, ícone de app, file dialogs"]
    registry["crates.io / docs.rs<br/>Distribuição e documentação dos crates"]
    icons["Icon providers<br/>Lucide SVGs e SVGs custom locais compilados em build-time pela CLI"]

    dev -->|Programa com / depende de<br/>API Rust| nive
    user -->|Interage com o app construído com| nive
    nive -->|Construído sobre| iced
    iced -->|Renderiza, abre janelas e captura input via| os
    nive -->|Instala ícone de app / abre file dialogs<br/>objc2 / winres / rfd| os
    dev -->|cargo install nive-cli / cargo add nive| registry
    nive -->|nive icons sincroniza glifos de| icons

    classDef person fill:#f7f7f7,stroke:#666,color:#222;
    classDef system fill:#e8f1ff,stroke:#4b77be,color:#111;
    classDef external fill:#eee,stroke:#999,color:#333;
    class dev,user person;
    class nive system;
    class iced,os,registry,icons external;
```

## Notas

- **Domínio-agnóstico na fronteira:** o runtime nunca depende de tipos de domínio do app;
  clientes/serviços do produto são construídos no *bootstrap* e injetados em
  `Application::init`.
- **Dois consumidores humanos:** o *App Developer* (API/DX) e o *Usuário Final* (a UI
  renderizada). O roadmap equilibra ambos: DX para o dev, densidade/performance para o
  usuário.
- O acoplamento ao SO é fino e isolado em `platform/` (ícone de app, file picker). O
  workspace tem apenas **2 ocorrências de `unsafe`**: o FFI objc2 do ícone de app no macOS
  (`platform/app_icon.rs`) e um `transmute_copy::<(), Window>` no *program runner*
  (`application/program.rs`) que materializa a janela-unit de apps de janela única.
