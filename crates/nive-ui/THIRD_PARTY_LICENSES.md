# Third-Party Licenses

## Lucide Icons

The SVG icons bundled under `assets/icons/lucide/` and
`assets/icons/generated/lucide/`, and embedded via the `src/icons/` catalog
modules, are sourced from the
[Lucide](https://lucide.dev) icon project, distributed under the ISC License.

```
ISC License

Copyright (c) Lucide Contributors

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.
```

## Inter

The static `Inter-Regular.ttf` and `Inter-SemiBold.ttf` faces bundled under
`assets/fonts/inter/` (embedded via `src/fonts.rs` behind the `bundled-fonts`
feature) are built from the
[Inter](https://github.com/rsms/inter) typeface (v4.1), distributed under the
SIL Open Font License, Version 1.1. The SemiBold face's family/subfamily name
records were relabeled from the upstream `Inter SemiBold` distinct-family
naming to a single `Inter` family with a `SemiBold` weight, so font matching
can select it by `(family = "Inter", weight = Semibold)`; no glyph or outline
data was modified. See `assets/fonts/inter/OFL.txt` for the full license text.

## Geist Mono

The static `GeistMono-Regular.ttf` and `GeistMono-Medium.ttf` faces bundled
under `assets/fonts/geist-mono/` (embedded via `src/fonts.rs` behind the
`bundled-fonts` feature) are sourced from the
[Geist](https://github.com/vercel/geist-font) typeface project, distributed
under the SIL Open Font License, Version 1.1. The Medium face's
family/subfamily name records were relabeled from the upstream
`Geist Mono Medium` distinct-family naming to a single `Geist Mono` family
with a `Medium` weight, so font matching can select it by
`(family = "Geist Mono", weight = Medium)`; no glyph or outline data was
modified. See `assets/fonts/geist-mono/OFL.txt` for the full license text.
