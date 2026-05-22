---
name: Ven Design System
colors:
  surface: '#131313'
  surface-dim: '#131313'
  surface-bright: '#3a3939'
  surface-container-lowest: '#0e0e0e'
  surface-container-low: '#1c1b1b'
  surface-container: '#201f1f'
  surface-container-high: '#2a2a2a'
  surface-container-highest: '#353534'
  on-surface: '#e5e2e1'
  on-surface-variant: '#b9cacb'
  inverse-surface: '#e5e2e1'
  inverse-on-surface: '#313030'
  outline: '#849495'
  outline-variant: '#3a494b'
  surface-tint: '#00dbe7'
  primary: '#e1fdff'
  on-primary: '#00363a'
  primary-container: '#00f2ff'
  on-primary-container: '#006a71'
  inverse-primary: '#00696f'
  secondary: '#ecffe3'
  on-secondary: '#003907'
  secondary-container: '#13ff43'
  on-secondary-container: '#007117'
  tertiary: '#fff5f4'
  on-tertiary: '#690003'
  tertiary-container: '#ffd0ca'
  on-tertiary-container: '#c3000a'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#74f5ff'
  primary-fixed-dim: '#00dbe7'
  on-primary-fixed: '#002022'
  on-primary-fixed-variant: '#004f54'
  secondary-fixed: '#72ff70'
  secondary-fixed-dim: '#00e639'
  on-secondary-fixed: '#002203'
  on-secondary-fixed-variant: '#00530e'
  tertiary-fixed: '#ffdad5'
  tertiary-fixed-dim: '#ffb4aa'
  on-tertiary-fixed: '#410001'
  on-tertiary-fixed-variant: '#930005'
  background: '#131313'
  on-background: '#e5e2e1'
  surface-variant: '#353534'
typography:
  display-lg:
    fontFamily: Geist
    fontSize: 48px
    fontWeight: '700'
    lineHeight: '1.1'
    letterSpacing: -0.04em
  headline-md:
    fontFamily: Geist
    fontSize: 24px
    fontWeight: '600'
    lineHeight: '1.2'
  body-base:
    fontFamily: Geist
    fontSize: 16px
    fontWeight: '400'
    lineHeight: '1.6'
  code-sm:
    fontFamily: JetBrains Mono
    fontSize: 14px
    fontWeight: '400'
    lineHeight: '1.5'
  terminal-output:
    fontFamily: JetBrains Mono
    fontSize: 13px
    fontWeight: '500'
    lineHeight: '1.4'
    letterSpacing: 0.02em
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  unit: 4px
  gutter: 16px
  margin-mobile: 16px
  margin-desktop: 32px
  max-width: 1280px
---

## Brand & Style

This design system is engineered for "ven," a high-performance version manager. The aesthetic merges the raw, data-driven intensity of a terminal with the sophisticated depth of modern glassmorphism. It is designed to evoke a sense of absolute control and technical mastery.

The visual language focuses on **high-energy minimalism**: 
- **Glassmorphism**: Translucent surfaces that suggest a multi-layered architectural stack.
- **Subtle Glows**: Interactive elements emit a soft luminance (cyan/green) to guide the eye without causing visual fatigue.
- **Tactile Precision**: Every border and line is razor-sharp, mimicking a high-resolution CLI environment.

## Colors

The palette is rooted in a "Void Black" foundation to maximize contrast and reduce eye strain during long coding sessions.

- **Base**: Midnight Black (#050505) for the lowest layer, Deep Charcoal (#0A0A0A) for surfaces.
- **Electric Cyan (#00F2FF)**: The primary action and highlight color. Used for focus states, primary buttons, and progress indicators.
- **Terminal Green (#00FF41)**: Reserved for "Success," "Active Version," and "Stable" tags. It should feel like a classic phosphor monitor.
- **Conflict Red (#FF3B30)**: High-saturation red for merge conflicts, errors, and destructive actions.
- **Glass Stroke**: A subtle 1px border using `rgba(255, 255, 255, 0.1)` is used to define glass boundaries.

## Typography

This design system utilizes a dual-font strategy to balance legibility with technical character.

- **Geist**: Used for all UI chrome, headings, and instructional copy. It provides a clean, Swiss-inspired modernist look that feels "engineered."
- **JetBrains Mono**: Used for all version strings, file paths, terminal outputs, and CLI command snippets. 

**Formatting Rules**:
- Headlines should use "sentence case" but remain tight in tracking.
- Terminal elements should always use the monospace font with a slightly increased letter spacing for maximum clarity in dense data.

## Layout & Spacing

The design system employs a **4px grid system** for micro-spacing and a **12-column fluid grid** for macro-layout.

- **Fixed-Fluid Hybrid**: Content containers are centered with a max-width of 1280px.
- **Internal Spacing**: Components use 8px (2 units) or 16px (4 units) padding to maintain a compact, "pro-tool" density.
- **Terminal Viewports**: Terminal sections should ignore standard padding to bleed to the edges of their specific containers, maximizing the display of log data.

## Elevation & Depth

Depth is created through **Backdrop Blurs** rather than traditional shadows.

1.  **Level 0 (Floor)**: Midnight Black (#050505). No blur.
2.  **Level 1 (Card/Surface)**: Deep Charcoal (#0A0A0A) at 70% opacity. `backdrop-filter: blur(12px)`.
3.  **Level 2 (Modals/Popovers)**: Lighter Charcoal at 80% opacity. `backdrop-filter: blur(24px)`. 1px Cyan glow border (0.1 opacity).
4.  **Interactive Glow**: When hovered, primary elements emit a `0px 0px 15px rgba(0, 242, 255, 0.3)` outer glow.

## Shapes

The shape language is "Soft-Technical." We avoid the friendliness of overly rounded corners, opting instead for a precise, sharp-but-not-harsh feel.

- **Base Radius**: 4px (`rounded-sm`).
- **Large Components**: 8px (`rounded-lg`) for cards and main terminal windows.
- **Buttons**: Strictly 4px. Never use pills or circles unless it's for a status indicator (dot).

## Components

- **Buttons**: Primary buttons are Electric Cyan with black text. Use a "ghost" style for secondary actions—1px cyan border with a subtle hover fill.
- **Terminal Blocks**: Deep black background, 1px border, with a header bar containing "traffic light" controls (Mac-style) or simple window labels in JetBrains Mono.
- **Status Chips**: Use "Terminal Green" for `active` or `installed` versions. These should be monospace text with a 1px border of the same color.
- **Conflict Indicators**: When a version conflict occurs, the entire card stroke should pulse with a subtle Red (#FF3B30) glow.
- **Input Fields**: Dark backgrounds, no fill, 1px bottom border. On focus, the border transitions to Electric Cyan with a subtle 2px glow.
- **Version Tree**: Vertical lines connecting versions should be 1px wide, using `rgba(255, 255, 255, 0.15)`.