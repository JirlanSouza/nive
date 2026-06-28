# Design System — Theme & Catálogo de Widgets

Como o design system de `nive-ui` está estruturado: tokens primitivos → tema semântico por
*roles* → specs concretos → widgets.

---

## 1. Modelo do Theme (diagrama de classes)

`Theme` resolve **roles semânticos** (intenção: "superfície de painel", "texto mudo",
"borda de foco") em **specs concretos** (cor, borda, sombra). Os widgets nunca leem cores
cruas — pedem por role.

```mermaid
classDiagram
    class Theme {
        <<enumeration>>
        Light
        Dark
        Custom
        +data() ThemeData
        +is_dark() bool
        +surface(SurfaceRole) SurfaceSpec
        +text(TextRole) TextSpec
        +border(BorderRole) BorderSpec
        +control(ControlRole, ControlState) ControlSpec
        +tone(ToneRole) ToneSpec
        +typography(TypographyRole) TextStyle
        +shape(ShapeRole) ShapeSpec
        +space(SpaceStep) f32
        +gap(GapRole) f32
        +padding(PaddingRole) Padding
        +control_metrics(ControlSize) ControlMetrics
    }
    class ThemeData {
        +name: str
        +mode: ThemeMode
        +color_scheme: ColorScheme
        +typography: TypographyScale
        +shapes: ShapeScale
        +spacing: SpacingScale
        +controls: ControlMetricsScale
    }
    class ColorScheme {
        +surface(role) SurfaceSpec
        +text(role) TextSpec
        +border(role) BorderSpec
        +control(role, state) ControlSpec
        +tone(role) ToneSpec
    }
    class SurfaceSpec {
        +background: Color
        +foreground: Color
        +border: BorderSpec
        +shadow: Shadow
    }
    class ControlSpec {
        +background: Color
        +foreground: Color
        +border: BorderSpec
        +focus: BorderSpec
    }
    class ToneSpec {
        +color: Color
        +on_color: Color
        +container: Color
        +on_container: Color
        +border: BorderSpec
    }
    class BorderSpec {
        +color: Color
        +width: f32
    }
    class ThemeBuilder {
        +new(name, mode)
        +palette(Palette)
        +primary/success/warning/danger(Color)
        +typography/shapes/spacing/controls(...)
        +build() Theme
        +build_data() ThemeData
    }
    class ThemeCatalog {
        +resolve(ThemeMode) Theme
        +get(ThemeId) ThemeData
    }

    Theme --> ThemeData : data()
    ThemeData *-- ColorScheme
    ColorScheme ..> SurfaceSpec : surface(role)
    ColorScheme ..> ControlSpec : control(role,state)
    ColorScheme ..> ToneSpec : tone(role)
    SurfaceSpec *-- BorderSpec
    ControlSpec *-- BorderSpec
    ToneSpec *-- BorderSpec
    ThemeBuilder ..> ThemeData : build_data()
    ThemeCatalog ..> Theme : resolve(mode)
```

## 2. Resolução por Role (token → role → spec → pixel)

```mermaid
flowchart LR
    tokens["tokens (const)<br/>color · spacing · radius<br/>shadow · typography"] -->|alimentam| theme
    role["Role semântico<br/>SurfaceRole · TextRole · BorderRole<br/>ControlRole+ControlState · ToneRole"] --> theme["Theme.accessor(role)"]
    theme --> spec["Spec concreto<br/>SurfaceSpec · TextSpec · BorderSpec<br/>ControlSpec · ToneSpec"]
    spec --> widget["Widget aplica<br/>background · foreground · border · shadow"]
```

`ControlState` (enabled · selected · `InteractionState{hovered,pressed,focused,dragged}`)
permite que um único role de controle cubra todos os estados visuais sem ramificação manual
no widget.

> **Lacuna do roadmap:** hoje `tokens::color` é hex/RGB. A migração para **OKLCH** (M0) entra
> exatamente nesta camada de tokens, sem mexer na API de roles acima.

---

## 3. Catálogo de Widgets (40+)

Todos são `nive-ui` puros (dependem só de `iced`), type-safe e estilizados por role.

```mermaid
flowchart TB
    subgraph inputs["Inputs & Formulários"]
        Input
        InputGroup
        Field
        Checkbox
        Switch
        Select
        SegmentedControl
        Autocomplete
        ColorInput
        ColorPicker
        PathInput
    end
    subgraph actions["Botões & Ações"]
        Button
        Pressable
        Toolbar
        DropdownMenu
        ActionCard
    end
    subgraph data["Dados & Display"]
        Text
        Icon
        Badge
        Card
        MetricCard
        KeyValueList["KeyValueList / DataRow"]
        ColorSwatch
        InitialAvatar
        VersionBadge
        Separator
    end
    subgraph layout["Contêineres & Layout"]
        Panel
        SplitPane
        Tabs["TabBar / TabItem"]
        TreeItem["TreeItem / OutlineTreeItem"]
        SectionHeader
        SelectableCard
        SelectableItem
    end
    subgraph overlays["Overlays"]
        Dialog
        Popover
        Tooltip
        CommandPalette["command_palette"]
    end
    subgraph feedback["Feedback & Estado"]
        Callout
        InlineAlert
        ProgressBar
        Spinner
        LoadingIndicator
        Skeleton
        EmptyState
        StatusLines["Error / Resource / Operation StatusLine"]
    end
    subgraph motion["Motion (determinístico)"]
        Animation["Animation · AnimatedLayout · AnimatedVisual · StaggeredPulse · Easing"]
    end
```

**Hosts & templates** (na raiz de `nive-ui`, compõem os widgets acima):
`DialogHost` (modal) · `ToastHost` (toasts) · `BootstrapView` (splash) · `focus_trap`
(ciclo de foco em overlays).

> **Lacunas do roadmap (Fase 2):** o catálogo cobre apps gerais e densos *exceto* os dois
> widgets analíticos pesados — **tabela virtualizada** e **gráfico de série temporal** — que
> entram como features opt-in (`tables`, `charts`).
