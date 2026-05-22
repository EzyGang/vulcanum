# Agent Instructions

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

Service files live in `src/services/<domain>/`. Each exports pure async functions that call `utils/api/request.ts` methods and return typed responses matching main-app models.

```typescript
import { get, post } from "../../utils/api/request";
import type { ProjectConfig } from "../../types/projects";

export const listProjects = () => get<ProjectConfig[]>("/projects");
export const createProject = (input: CreateProjectRequest) => post<ProjectConfig>("/projects", input);
```

### Error Handling

Backend returns errors as `{ error: "message" }`. The `ApiError` class parses this body:
```typescript
try {
  await listProjects();
} catch (e) {
  if (e instanceof ApiError) {
    e.status   // HTTP status code
    e.message  // error string from body
  }
}
```

## State Strategy

| State Type | Tool | Scope |
|------------|------|-------|
| Server state | `@tanstack/react-query` via `useApiQuery`/`useApiMutation` | Cached, refetched, invalidated |
| Client-only UI state | `@preact/signals` stores | Theme, sidebar, auth token |
| Component-local state | `useSignal` | Ephemeral form state, toggles |

Never cache the same data in both signals and React Query. Server data lives in React Query only.

## Auth Flow

Magic-link authentication:
1. `POST /api/v1/auth/login` with `{ email }`
2. `GET /api/v1/auth/verify?token=...` returns `{ message, user: { id, email } }`
3. Store the returned token in `stores/auth.store.ts`

## Directory Map

| Directory | Holds |
|-----------|-------|
| `components/<feature>/ui/` | Presentational views (`.view.tsx`) |
| `components/<feature>/hooks/` | Data hooks (`.hook.ts`) |
| `components/<feature>/containers/` | Glue containers (`.container.tsx`) |
| `pages/` | Route-level page components |
| `services/` | API call functions wrapping `utils/api/request.ts` |
| `stores/` | Global signal-based stores |
| `hooks/` | Shared generic hooks |
| `utils/` | Cross-feature utilities (API client, case conversion) |
| `routes/` | Route definitions |
| `types/` | TypeScript type definitions |

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
