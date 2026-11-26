# Design

EnvVault uses a sharp, flat, technical, monochrome visual language. Reference points: Better Auth docs, Vercel/Geist dashboard feel, and default shadcn/ui with zero radius.

## Rules

- **No rounded corners anywhere.** `--radius: 0rem` in `src/index.css`. No `rounded-*` Tailwind classes. Override shadcn components if they hardcode rounding.
- **Borders, not shadows.** Separate areas with 1px solid borders (`border-border`). No box shadows, no gradients, no glassmorphism, no blur.
- **Monochrome palette.** Near-black/near-white backgrounds and neutral grays for borders/muted text. Accent color only for meaning: red destructive/errors, green success, amber warnings. Primary buttons are solid foreground-on-background.
- **Typography.** Geist Sans for UI; Geist Mono for keys, values, environment names, code, and keyboard shortcuts. Fonts are bundled locally (no CDN). Base size 13-14px, medium labels, slightly tight heading tracking.
- **Density.** Compact 32-36px table rows, small paddings, 1px dividers. Information-dense but not cramped.
- **Dark mode first.** Light mode follows the system theme. Both must be polished. Theme tokens are CSS variables in `src/index.css`.
- **Icons.** `lucide-react`, 16px, 1.5px stroke.
- **Motion.** Minimal. 100-150ms opacity/color transitions only. No bouncy or scale animations.
- **Focus states.** Visible 1px/2px square ring; never a rounded glow.
- **Details.** Uppercase small muted section labels (e.g. `ENVIRONMENTS`), square `kbd`-style hints, thin grid lines/faint dotted background only on empty states.

## Layout

- Two panes: fixed ~260px sidebar (project > environments) separated by a 1px border from main panel.
- Main header: breadcrumb (`project / environment`), variable count, actions (Import, Copy as .env, Export).
- Variable table columns: Key (mono) | Value (mono, masked `••••••••`) | Comment | Actions (reveal, copy, delete). Inline editing on click.
- Empty states: short muted text, one primary button, keyboard hint.
- Dialogs: square, 1px border, plain dimmed backdrop (no blur).

## Theme setup

The theme is defined in `src/index.css` using Tailwind CSS v4 tokens and CSS variables. Keep all colors as variables for light and dark so the app can be re-themed from one file.
