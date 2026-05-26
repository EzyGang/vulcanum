# Agent Instructions

Project-wide Reference Guide
(ingest once; treat as implicit context for every future discussion)

Playwright-cli skill artifacts should all live inside the `.playwright-cli` folder!

## Tech Stack

- TypeScript / Preact 10 + @preact/signals (state)
- Vite build / Tailwind CSS v4 in JSX (`@import "tailwindcss"` in CSS)
- Routing: wouter-preact

## Developer Commands

```bash
pnpm dev              # Dev server at http://localhost:5173
pnpm build            # Production build → dist/
pnpm preview          # Preview production build

pnpm validate         # Lint + type-check (ci-run ready)
pnpm fix              # Format + type-check
pnpm lint             # Biome check
pnpm format           # Biome check --write
pnpm type-check       # tsc --noEmit

pnpm test             # Vitest run
pnpm test:watch       # Vitest watch mode
```

## Directory Structure (src/)

```
components/      feature-oriented UI (strict triplet pattern)
pages/           thin wrappers rendering one container
hooks/           shared generic hooks
stores/          global singletons using @preact/signals
services/        API endpoint helpers (pure request/response)
utils/           cross-feature helpers
routes/          public/protected route definitions
types/           TypeScript type definitions
```

## Component Pattern (Strict Triplet)

Every feature uses exact naming:

- `hooks/useFeature.hook.ts` – data fetching, side-effects, signals
- `ui/Feature.view.tsx` – pure presentational JSX, zero side-effects
- `containers/Feature.container.tsx` – glue: hook → view (≤20 lines)

### Rules

• Exact naming: `Foo.view.tsx`, `useFoo.hook.ts`, `Foo.container.tsx`.
• View: zero side-effects, zero inline handlers, zero logic. Max 150 lines.
• Hook: all side-effects (service calls, signals, React Query). Pre-bind all callbacks here.
• Container: prop passthrough only. Group props into logical objects: `{ data, status, actions }`.
• Page component in `src/pages` renders a single container + routing / error fallback.
• Max 6-8 props passed from Container → View. If more, refactor to use Context.
• Views consuming Context have ZERO props - all data comes from `useContext()` hooks.

Pages in `src/pages/` render a single container + routing/error handling.

## Key Conventions

- Strict TypeScript (no implicit any)
- Biome for lint/format (see `.biome.json`)
- Tailwind v4 for all styling; no CSS-in-JS
- React compatibility via `preact/compat` alias
- Use `createContext` from preact to avoid prop drilling with signals

## Design System

### Visual Design Reference — Dark-First Minimalism

All theme colors are defined with **OKLCH** in `style.css`. Do not use HEX or RGBA in theme tokens.

**1. Overall Aesthetic**
- Dark-first aesthetic with pure black body background.
- Deep charcoal surfaces for cards and panels.
- Clean, high-contrast white typography on dark backgrounds.
- Minimal, border-heavy — cards/panels use faint white-opacity borders.
- Never use heavy shadows or gradients; rely on borders and background-layers for depth.

**2. Typography**
- **Sans-serif:** `Inter` is our main UI typeface (regular, medium, semibold, bold weights).
- **Monospace:** `JetBrains Mono` is our code/mono typeface.
- Hero headings should be large (32–72px depending on breakpoint), tight line-height (~1.05–1.1), clean geometric sans-serif.
- Body text should be `1rem` / `16px`, line-height `1.5`, colored at `oklch(100% 0 0 / 0.8)`.
- Use uppercase only for small labels, nav, and CTAs (letter-spacing wide `0.05em`).

**3. Color Palette**
- **Background:** `oklch(0% 0 0)` (primary), `oklch(14% 0 0)` (surface), `oklch(20% 0 0)` (card panels), `oklch(22% 0 0)` (subtle panels), `oklch(23% 0 0)` (raised).
- **Text:** `oklch(100% 0 0)` (primary), `oklch(100% 0 0 / 0.8)` (secondary), `oklch(65% 0 0)` (muted).
- **Borders:** `oklch(100% 0 0 / 0.1)` or `oklch(26% 0 0)`.
- **Accents:**
  - Green: `oklch(82% 0.2 145)` (success / availability highlight).
  - Emerald: `oklch(78% 0.18 165)` (soft secondary accent).
  - Red: `oklch(55% 0.2 25)` (error / warnings).
  - Orange: `oklch(65% 0.2 45)` (tertiary highlight).
- **Hover:** Surfaces lift slightly (`oklch(100% 0 0 / 0.12)` for hover backgrounds), borders may brighten to `oklch(100% 0 0 / 0.25)`.

**4. Buttons**
- **Primary (CTA):**
  - Background: `oklch(100% 0 0)` (pure white) on dark.
  - Text: `oklch(0% 0 0)` (black).
  - Padding: `12px 16px` (roughly `px-4 py-3`).
  - Font-size: `0.875rem` (`text-sm`), uppercase, wide letter-spacing.
  - Border-radius: `0px` (sharp / square edges).
  - Hover: subtle opacity reduction (`opacity: 0.9`) or slight darkening of the button only.
- **Secondary / Ghost:**
  - Background: `transparent`.
  - Border: `1px solid oklch(100% 0 0 / 0.1)`.
  - Text: `oklch(100% 0 0)`.
  - Hover: background `oklch(100% 0 0 / 0.12)`, border brightens.
- **Links:**
  - White text, no underline by default.
  - Underline and opacity change on hover.

**5. Spacing**
- Generous padding in sections (vertical gaps of `80px`–`120px` on large screens).
- Use flexbox with `gap` for all internal spacing (never `space-y-*` / margins on children).
- Card padding: roughly `20px` (`p-5`).
- Nav height: compact, minimal padding, transparent until scrolled.

**6. Components**
- **Cards:** Use `bg-surface`, `border border-white/10`, no radius (or `0.25rem` absolute max). No box-shadow.
- **Inputs:** `bg-surface`, border `oklch(100% 0 0 / 0.12)`, focus border `oklch(100% 0 0 / 0.5)`, subtle monospace feel.
- **Nav:** Transparent background, white text links, uppercase small labels, sharp logo text.
- **Tags / Badges:** Very small (`text-xs`), dark background, white-opacity border, uppercase.

**7. Tailwind v4 Mapping (to keep in `@theme`)**
- `bg-surface` → `oklch(14% 0 0)`
- `bg-surface-card` → `oklch(20% 0 0)`
- `bg-surface-raised` → `oklch(23% 0 0)`
- `text-white` / `text-white/80` / `text-white/70`
- `border-white/10` / `border-white/12`
- `hover:bg-white/12` / `hover:border-white/25`

### Layout Pattern (Flexbox Standard)

**Always use flexbox with gap for spacing. Never use `space-y-*` or margins for sibling spacing.**

```tsx
// Vertical stack - CORRECT
<div class='flex flex-col gap-4'>
  <Item />
  <Item />
</div>

// Horizontal row - CORRECT
<div class='flex items-center gap-2'>
  <Icon />
  <Text />
</div>

// Centered header - CORRECT
<div class='flex flex-col items-center gap-4 text-center'>
  <Icon />
  <Title />
  <Subtitle />
</div>
```

**Parent controls spacing, not children.** Define all gaps at container level.

### Theme System

Three-mode theme toggle (system/light/dark) with localStorage persistence:

- `themeModeSignal`: Current mode ('system' | 'light' | 'dark')
- `effectiveThemeSignal`: Computed effective theme
- `cycleThemeMode()`: Cycle through modes
- `setThemeMode(mode)`: Set specific mode

Uses `data-theme` attribute on `<html>` for Tailwind dark mode.

## Stores Pattern

Plain objects with signals - no classes, no getters/setters:

```typescript
import { signal } from "@preact/signals";

interface FeatureStore {
  data: Signal<T[]>;
  loading: Signal<boolean>;
}

export const featureStore: FeatureStore = {
  data: signal([]),
  loading: signal(false),
};

// Use directly: featureStore.data.value = newData
```

## API Layer

### Base URL

- Dev: `VITE_API_URL` env var or `http://localhost:8000`
- Prod: `/api/v1`

### Client

`utils/api/client.ts` exports `fetchApi<T>(path, options)` — the single internal fetch wrapper:

- Prepends base URL
- Injects `Authorization: Bearer <token>` from `stores/auth.store.ts`
- Converts request body keys to `snake_case`, response keys to `camelCase`
- Throws `ApiError { status, serverError }` on non-2xx responses (parses `{ error: string }` from body)
- In dev, logs request/response unless `VITE_DISABLE_DEV_LOGGING` is set

`utils/api/request.ts` exports typed wrappers: `get`, `post`, `put`, `patch`, `del`.

`utils/api/query/client.ts` exports a shared `QueryClient` instance and an `invalidate` helper.

`utils/api/query/hooks.ts` exports `useApiQuery` and `useApiMutation` wrappers with `ApiError` typed errors.

### Service Files

Service files live in `src/services/<domain>/`. Each exports pure async functions that call `utils/api/request.ts` methods and return typed responses matching server models.

```typescript
import { get, post } from "../../utils/api/request";
import type { ProjectConfig } from "../../types/projects";

export const listProjects = () => get<ProjectConfig[]>("/projects");
export const createProject = (input: CreateProjectRequest) =>
  post<ProjectConfig>("/projects", input);
```

### Error Handling

Backend returns errors as `{ error: "message" }`. The `ApiError` class parses this body:

```typescript
try {
  await listProjects();
} catch (e) {
  if (e instanceof ApiError) {
    e.status; // HTTP status code
    e.message; // error string from body
  }
}
```

## State Strategy

| State Type            | Tool                                                       | Scope                          |
| --------------------- | ---------------------------------------------------------- | ------------------------------ |
| Server state          | `@tanstack/react-query` via `useApiQuery`/`useApiMutation` | Cached, refetched, invalidated |
| Client-only UI state  | `@preact/signals` stores                                   | Theme, sidebar, auth token     |
| Component-local state | `useSignal`                                                | Ephemeral form state, toggles  |

Never cache the same data in both signals and React Query. Server data lives in React Query only.

## Auth Flow

Magic-link authentication:

1. `POST /api/v1/auth/login` with `{ email }`
2. `GET /api/v1/auth/verify?token=...` returns `{ message, user: { id, email } }`
3. Store the returned token in `stores/auth.store.ts`

## Directory Map

| Directory                          | Holds                                                 |
| ---------------------------------- | ----------------------------------------------------- |
| `components/<feature>/ui/`         | Presentational views (`.view.tsx`)                    |
| `components/<feature>/hooks/`      | Data hooks (`.hook.ts`)                               |
| `components/<feature>/containers/` | Glue containers (`.container.tsx`)                    |
| `pages/`                           | Route-level page components                           |
| `services/`                        | API call functions wrapping `utils/api/request.ts`    |
| `stores/`                          | Global signal-based stores                            |
| `hooks/`                           | Shared generic hooks                                  |
| `utils/`                           | Cross-feature utilities (API client, case conversion) |
| `routes/`                          | Route definitions                                     |
| `types/`                           | TypeScript type definitions                           |

<very_important_block>

- \*.view.tsx SHOULD ONLY CONTAIN STYLES AND LAYOUT NO LOGIC/FUNCTIONS/VARIABLES DEFINITION THERE.
- MUST FOLLOW `DRY` (DO NO REPEAT YOURSELF) principle, NO code repetition should exist in everything you do for ANY reason, unless ther is really no way around it.
- Do not create `index.ts` files for re-exports.
- Use `export const foo = () = {}`, don't put the export list at the bottom of the file.
- Don't use `function App()` use `const App = () => {}` style of definitions.
- Use `class=""` in preact for classes. Only use `class=""` in very specific edge cases.
- This project uses `@preact/signals` based reactivity everywhere. Meaning that global states/signals use `const signalName = signal()` and everything component scoped should use `useSignal` hooks. YOU SHOULD NOT CREATE THE STATES using `useState`!
- REUSE INSTEAD OF REIMPLEMENTING, IF UNSURE WHETHER SOMETHING EXIST - SEARCH FOR IT AND ONLY IF IT DOESN'T IMPLEMENT.
- In TS use `for...of` instead of `forEach`.
  <bad_example>

```ts
objs.forEach((obj) => console.log(JSON.stringify(obj)))'
```

</bad_example>
<good_example>

```ts
for (const obj of objs) {
  console.log(JSON.stringify(obj));
}
```

</good_example>

# Code guide

You should aim to write simple, readable, and maintainable code. Over-abstracting stuff and overcomplicating is not endorsed, remember the less code - the better, as code is a liability.

</very_important_block>

## Testing

Tests should live in a `src/tests` folder.

Use vitest when implementing these.
