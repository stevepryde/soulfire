# Soulfire UI Design Language

Status: implementation guide for the Tauri + React port. The normative UI contract remains
[`specs/09-ui.md`](specs/09-ui.md); this file translates Soulfire-OG's concrete visual system into
React/CSS rules so parity work stays anchored.

## Source Of Truth

| Layer | Source | Rule |
| --- | --- | --- |
| Product contract | `specs/09-ui.md` | Defines required surfaces and OG fidelity. |
| Visual reference | `~/projects/app-world/soulfire/bins/soulfire-ui` | Match layout, tokens, component shape, and interaction tone unless a spec deliberately adapts it. |
| React implementation | `src/`, `src/styles.css` | May rename implementation classes, but should preserve OG geometry and behavior. |

## Palette

Soulfire is dark-first. The default page should read closer to OG's black/purple glass surface than
to a brown admin console.

| Role | OG Reference | React Token |
| --- | --- | --- |
| Page background | `#0f0f0f`, top `primary/30` glow, black vertical gradient | `--color-background`, `--bg` |
| Surface | `#1a1a1a`, `#1b1b1e`, `rgba(255,255,255,0.03)` | `--color-surface`, `--bg-panel` |
| Elevated card | black glass with `border-white/8`, 16-36px shadow | `.surface-card`, `.list-panel`, `.settings-card` |
| Primary accent | Theme variable, purple default `#8b5cf6` | `--color-primary`, `--primary-base` |
| Secondary accent | Link teal `#00F5D4` | `--color-link`, `--teal` |
| Text | White primary, `#B3B3CC` secondary | `--text`, `--text-muted` |
| Danger | Pink/rose, never browser-native dialogs | `--rose` |

## Typography

| Role | Font | Weight | Size Guidance |
| --- | --- | --- | --- |
| Wordmark | Merriweather serif | 700 | 30-36px in titlebar, gradient fire text. |
| Page heading | Inter/system sans | 600 | 24px on list pages, 20-24px in panel headers. |
| Section eyebrow | Inter/system sans | 600 | 11px uppercase, `0.24em` tracking, primary-light. |
| Body | Inter/system sans | 400 | 14-16px. |
| Narrative/adventure prose | Merriweather serif | 400-700 | Use for immersive play/chat prose once those screens land. |

Letter spacing stays normal except OG-style small uppercase eyebrows. Do not scale fonts with viewport
width.

## Layout

| Surface | OG Shape | React Rule |
| --- | --- | --- |
| Standard pages | Full-height dark page, fixed 64px translucent titlebar, content starts at `pt-20` | Keep top titlebar fixed/visually dominant; content width maxes at 70rem for Worlds, 4xl for Characters-like lists. |
| Background | Black vertical gradient plus primary top glow and two blurred accents | Use page-level pseudo/background layers, not unrelated decorative blobs. |
| Worlds home | Centered tab row, search bar, 16:6 world/adventure cards, two-column desktop grid | Prefer tabbed sections and card media before plain list panels. |
| Characters list | Header with create actions, search, stacked full-width character rows | Prefer avatar + text rows over square generic cards. |
| Settings | Stacked rounded cards max-width 3xl, `rounded-[24px]`, border-white/8 | Keep settings dense, grouped, and carded like OG settings. |
| Immersive screens | No app chrome; backdrop image/theme, floating header pill, glass composer | Do not build play/chat inside the standard shell. |

## Components

| Component | OG Contract | Implementation Guidance |
| --- | --- | --- |
| Titlebar | Fixed top, logo left, centered fire-gradient wordmark, profile/status right | React titlebar should move toward this instead of sidebar-brand dominance. |
| Buttons | Rounded full or 12px rounded, primary uses theme accent; secondary uses white/10 border | Avoid boxy utility/admin buttons. |
| Search | Rounded 14px dark field with left search icon and clear button | Search should not require a separate text "Search" button when live/debounced search is possible. |
| Cards | World/adventure cards have 16:6 media, overlay gradient, title/description/actions | Use cards for repeated content, not nested cards inside page sections. |
| Character rows | Round 56-64px portrait/emoji, name, subtitle/detail, timestamp/status, row actions | Make the whole row clickable. |
| Dialogs | In-app confirmation modal with product styling | Never use native `confirm`, `alert`, or `prompt`. |
| Option pickers | Custom dropdown/button grid/swatch controls | Never use native `<select>`. |
| Tabs | Centered rounded buttons; active state uses primary accent | Worlds-style sections should use tabs instead of side-by-side unrelated panels. |

## Spacing And Radius

| Token | Value | Use |
| --- | --- | --- |
| `space-1` | 4px | Tight icon/text gaps. |
| `space-2` | 8px | Small control gaps. |
| `space-3` | 12px | Form row gaps. |
| `space-4` | 16px | Card internal rhythm. |
| `space-5` | 20px | Page section gaps. |
| `space-6` | 24px | Large card padding. |
| Small radius | 12-14px | Search fields, compact controls. |
| Card radius | 20-30px | OG glass cards and empty/error states. |
| Avatar/media radius | Full for portraits; 18-22px for 16:6 media | Preserve OG image framing. |

The current React app has several 7-8px admin-style radii. Future UI parity work should migrate
standard-page cards and searches toward the OG radii above while preserving usability.

## Accessibility

| Area | Rule |
| --- | --- |
| Touch targets | Minimum 44px for primary navigation, row actions, destructive actions, and mobile controls. |
| Focus | Visible focus rings using primary or teal accent, never outline removal without replacement. |
| Text contrast | Primary text on dark surfaces is white; secondary text stays at or above OG `white/52` contrast. |
| Rows | Full repeated rows/cards are clickable when they navigate or select. |
| Dialogs | `role="dialog"`, `aria-modal`, labelled heading, keyboard reachable buttons. |

## Motion

Use restrained OG-like motion: 150-200ms hover/active transitions, subtle card hover lift where OG
uses it, and respect reduced-motion once animations are introduced. Streaming/play surfaces can have
typing/status motion; settings and list screens should stay calm.

## Current React Drift To Correct

| Drift | Correction |
| --- | --- |
| Sidebar-first app chrome | Move toward OG fixed titlebar plus mobile bottom nav / standard-page content. |
| Brown/ember-heavy shell palette | Restore dark black/purple base with themed primary glow; reserve fire gradient for wordmark/onboarding. |
| Plain list panels for Worlds | Move toward OG tabbed Worlds home and 16:6 adventure/world cards. |
| Character square cards | Move toward OG full-width chat rows with portraits and action affordances. |
| Search submit buttons | Move toward OG live/debounced search with icon and clear action. |
| Low-radius admin controls | Use OG 12-30px radii depending on component role. |

## Verification

Every UI parity milestone should run:

- `bun run build`
- `bun run test:ui` when React behavior changes
- guardrail scan for native `confirm`, `prompt`, `alert`, `<select>`, and browser storage
- visual/manual check against `docs/MANUAL_SMOKE.md` when a surface is user-visible
