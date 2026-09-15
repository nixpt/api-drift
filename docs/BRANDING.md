# api-drift Brand & Visual Identity Guidelines

Status: Approved (2026-09-14).

## 1. Identity & Philosophy

**api-drift** provides contract drift detection, break classification, and mechanical patch suggestion for Rust crates and software interfaces.

When a shared crate or interface moves an API (a renamed function, a new required argument, or a removed type), every downstream consumer breaks. api-drift provides the policy and type layer for that loop: **what changed, how bad is it, what edit fixes it** — as structured data an agent, a script, or a human can act upon.

The visual identity embodies this core mission:
- **Baseline Stability vs. Mutational Drift:** The tension between the established contract (v1 baseline) and the evolving implementation (v2 drift).
- **Classification Precision:** The three-tier severity hierarchy (Compatible, Warning, Breaking) rendered as dedicated differential spectral channels.
- **Automated Bridge:** The transition from detected breakage to automated suggestion (`auto_appliable` patch vectors).

---

## 2. Core Visual Elements

### The Emblem: `⟨ Δ ⟩` (The Differential Drift Bracket)

```text
     /|            ▲             |\
    / |    .......[Δ].......>    | \
   <  ●   ------------------>    |  ★ >
    \ |   ==================>    | /
     \|                         |/
```

- **Symbolism**:
  - **Left Bracket `⟨` (Electric Azure / `#38bdf8`)**: Represents the established **v1 Baseline Public Contract**. An immutable, stable anchor coordinate with a precise vertex node.
  - **Right Bracket `⟩` (Drift Coral / `#f43f5e`)**: Represents the **v2 Mutated Public Surface**. Displaced along a drift vector upwards and outwards, leaving behind a dashed phantom footprint of where the API was previously anchored.
  - **Central Delta `Δ` (Differential Core / `#818cf8`)**: Represents the mathematical and semantic divergence between version snapshots.
  - **The Three Severity Channels**:
    1. **Top Channel (`#10b981` Emerald)**: Compatible changes (widened traits, pure additions).
    2. **Mid Channel (`#f59e0b` Solar Amber)**: Warning changes (struct field or enum variant additions under exhaustive types).
    3. **Bottom Channel (`#f43f5e` Crimson ──▶ `#38bdf8` Azure)**: Breaking changes (signature edits, removed items) bridged forward by the mechanical suggestion patch arrow.

### The Wordmark

The canonical wordmark is set with a clean sans title paired with block and monospace elements:

```text
  █████  ████   █████         ████   ████   █████  █████  █████
  █   █  █   █    █   ███████ █   █  █   █    █    █        █  
  █████  ████     █   ███████ █   █  ████     █    ████     █  
  █   █  █        █           █   █  █  █     █    █        █  
  █   █  █      █████         ████   █   █  █████  █        █  
```

---

## 3. Color Palette

The `api-drift` color system is calibrated for deep void backgrounds (OLED, modern terminal emulators, dark documentation themes) with high-contrast functional highlights.

| Role | Color Name | Hex | RGB | ANSI 24-bit Escape | Usage |
|---|---|---|---|---|---|
| **Base Canvas** | Obsidian Void | `#070a12` | `(7, 10, 18)` | `\e[48;2;7;10;18m` | Background canvas |
| **Surface Dark** | Dark Slate | `#0f172a` | `(15, 23, 42)` | `\e[48;2;15;23;42m` | Cards, panel backgrounds, badges |
| **Surface Border**| Muted Steel | `#1e293b` | `(30, 41, 59)` | `\e[48;2;30;41;59m` | Structural borders and dividers |
| **Primary Baseline** | Electric Azure | `#38bdf8` | `(56, 189, 248)` | `\e[38;2;56;189;248m` | Baseline contract, title accent, active nodes |
| **Cyber Accent** | Neon Cyan | `#00f0ff` | `(0, 240, 255)` | `\e[38;2;0;240;255m` | Highlights, suggestion arrows, glow nodes |
| **Compatible / Fix** | Emerald Mint | `#10b981` | `(16, 185, 129)` | `\e[38;2;16;185;129m` | Source-compatible, auto-appliable edits |
| **Warning** | Solar Amber | `#f59e0b` | `(245, 158, 11)` | `\e[38;2;245;158;11m` | Warning severity, review flags |
| **Breaking / Drift** | Crimson Ruby | `#f43f5e` | `(244, 63, 94)` | `\e[38;2;244;63;94m` | Breaking changes, drift vector |
| **Foreground Ice** | Pure Ice | `#f8fafc` | `(248, 250, 252)` | `\e[38;2;248;250;252m` | Primary headings, wordmark |
| **Foreground Muted**| Silver Slate | `#94a3b8` | `(148, 163, 184)` | `\e[38;2;148;163;184m` | Subtitles, labels, secondary metadata |

---

## 4. Asset Inventory

All canonical branding assets are version-controlled in [`assets/branding/`](../assets/branding/):

| Asset | Format | Resolution / Size | Description & Best Use |
|---|---|---|---|
| [`api-drift-banner.svg`](../assets/branding/api-drift-banner.svg) | SVG | 1200 × 420 | Vector hero banner with chrome navigation bar, glowing differential emblem, metadata pills, and architectural pipeline diagram. |
| [`api-drift-banner-1200.png`](../assets/branding/api-drift-banner-1200.png) | PNG | 1200 × 420 | Master raster banner for GitHub README headers, social sharing cards (Open Graph), and crates.io/docs.rs documentation. |
| [`api-drift-logo.svg`](../assets/branding/api-drift-logo.svg) | SVG | 860 × 220 | Horizontal logo lockup with standalone emblem, wordmark, badges, tagline, and mini pipeline indicators. |
| [`api-drift-logo.png`](../assets/branding/api-drift-logo.png) | PNG | 860 × 220 | Master raster logo for documentation sidebars and headers. |
| [`api-drift-icon.svg`](../assets/branding/api-drift-icon.svg) | SVG | 512 × 512 | Vector square app icon with squircle container, blueprint grid, ambient backlight, and glowing differential emblem. |
| [`api-drift-icon-512.png`](../assets/branding/api-drift-icon-512.png) | PNG | 512 × 512 | Master raster icon for desktop packaging, web avatars, and registry listing. |
| [`api-drift-icon-128.png`](../assets/branding/api-drift-icon-128.png) | PNG | 128 × 128 | Medium raster icon for notification hubs and toolbars. |
| [`api-drift-icon-64.png`](../assets/branding/api-drift-icon-64.png) | PNG | 64 × 64 | Small raster icon / favicon. |
| [`api-drift-icon-32.png`](../assets/branding/api-drift-icon-32.png) | PNG | 32 × 32 | Favicon / small status icon. |
| [`api-drift-mark.svg`](../assets/branding/api-drift-mark.svg) | SVG | 256 × 256 | Standalone differential emblem without container background for inline vector documentation. |
| [`api-drift-mark.png`](../assets/branding/api-drift-mark.png) | PNG | 256 × 256 | Standalone raster mark. |
| [`api-drift-banner.txt`](../assets/branding/api-drift-banner.txt) | UTF-8 | 7 rows | Plain text & Unicode block ASCII banner for CLI `--version` output, MOTDs, and help dialogs. |
| [`api-drift-ansi.txt`](../assets/branding/api-drift-ansi.txt) | ANSI | 24-bit TrueColor | Colorized terminal banner with gradient mapping. Output via `cat assets/branding/api-drift-ansi.txt`. |

---

## 5. Usage in Code and Documentation

### README Hero Header
Add to the top of `README.md`:

```markdown
<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/branding/api-drift-banner-1200.png">
    <img src="assets/branding/api-drift-banner-1200.png" alt="api-drift banner" width="100%">
  </picture>
</p>
```

### Terminal Banner
Displaying the TrueColor ANSI banner in terminal scripts or CLI tools:

```bash
cat assets/branding/api-drift-ansi.txt
```

### Regenerating Raster Assets
To regenerate all raster PNGs and terminal assets from the master SVGs:

```bash
python3 scripts/generate-branding-pngs.py
```
