# Agent instructions

Before choosing or changing an approach for this app, read these:

- [docs/requirements.md](docs/requirements.md) is the product spec.
- [docs/decision-context.md](docs/decision-context.md) is the architectural decision log. Read every entry. Log only architectural choices and constraints that explain the code, not individual fixes or UI tweaks (those go in the git log). When a decision changes an existing entry, update that entry and add a dated line to its **Revised** list; otherwise append a new dated entry (date, what was decided, why).

## Where work lives

Rust owns the heavy work: file conversion, process control, parsing, image preparation, and anything that encodes, decodes, or shells out. Vue, TypeScript, and HTML only render controls, previews, progress, and inspector data. The frontend calls Tauri commands and listens for events. It does not encode, decode, parse KTX, or build a command line.

## Encoder option defaults (no placeholders)

When `ktx create` (or this app) has a known default for an option, put that value in `CreateOptions::default` / the store as a real value. Do **not** leave the field unset and show the default as an input `placeholder`, `empty-label` like “Default (medium)”, or “tool default”.

This matters especially when other options’ enablement depends on the value being set (for example `scale: 1` unlocks mipmap filter controls). Placeholders are only for format hints on fields that truly have no default (for example swizzle pattern `rgba` when the flag is omitted). Unset (`null` / `None`) means “omit the flag”, not “pretend the default is visible”.

## Never offer known-invalid encode combinations

The GUI must **never** let the user select or keep a combination of sources and options that we already know will make `ktx create` (or our wrapper) fail. Hide or auto-correct those choices in the options rules engine; refuse encode as a backstop. Do not rely on the user reading a fatal from the sidecar. When the tool’s rules change (pin bump), update the filters with them.

## TypeScript

`strict` stays on. Write types the checker can prove.

- No `any`. No `as any`, `as unknown as`, double assertions, or other casts that skip checking. No `@ts-ignore` or `@ts-expect-error` to hide a real type hole.
- Prefer `satisfies` when a value must match a type and the narrow inferred type must stay. A type annotation widens the value. `satisfies` (TypeScript 4.9) only checks it.

```ts
const palette = {
  red: [255, 0, 0],
  green: "#00ff00",
} satisfies Record<"red" | "green" | "blue", string | [number, number, number]>;

palette.green.toUpperCase();
```

- Reach for generics, conditional types, mapped types, and `infer` before a cast. Add a small utility type in the project when the same shape is needed in more than one place.
- Model Rust command payloads as explicit types shared with the frontend. Do not widen them to `unknown` and cast on the way out.

## ESLint

Lint with the strict type-checked typescript-eslint configs (`strict` and `strictTypeChecked`), plus the Vue recommended flat config. A lint error is fixed in the code. Do not disable a rule, weaken the config, or add an inline disable to get a green run. The existing `vue/multi-word-component-names` exception stays.

## Styles

Style with UnoCSS utility classes (`presetWind3`, attributify, icons, directives, variant groups). Reach for a `<style lang="scss">` block only when a utility class cannot express the rule, such as a keyframe or a selector UnoCSS does not cover. Keep that SCSS local to the component.

## Reuse before writing

Before writing a helper, check whether it already exists:

1. [VueUse](https://vueuse.org/) (`@vueuse/core`) for Vue composition utilities: element size, event listeners, refs, async state, and similar.
2. Context7 (`resolve-library-id`, then `query-docs`) or a web search for an existing npm package.

Use the existing function or package when it covers the need. Write project code when nothing maintained does, or when pulling in a package would be larger than the few lines required.

## Fallow

Use `npx fallow` (devDependency). Prefer `--format json --quiet`.

- Before committing AI-generated changes: `fallow audit --format json --quiet`
- Cleanup / dupes / complexity: `fallow dead-code`, `fallow dupes`, `fallow health` (same JSON flags)
- Trace before deleting: `fallow dead-code --trace <file>:<export>` or `--trace-dependency <name>`
- Always `--dry-run` before `fallow fix`, then `fix --yes`

Run `npm run check` before every commit. Fallow findings (dead code, duplication, health) are fixed in the code, like lint errors. Do not disable a rule, weaken a threshold, or edit `.fallowrc.json` to get a green run. The only allowed suppression is the “written ahead of its consumer” marker described in [docs/development-plan.md](docs/development-plan.md) (`@expected-unused` / `fallow-ignore-file unused-file` naming the consuming task).

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/). The type drives the release: `feat` → minor, `fix` / `perf` → patch, `!` or a `BREAKING CHANGE:` footer → major; other types do not release. Not enforced by a commit hook; semantic-release is the consumer.

<!-- generated:task-matrix:start -->
| When the agent is about to... | Run |
|---|---|
| delete an "unused" export or file | `fallow dead-code --trace <file>:<export>` |
| prove a TypeScript symbol's exact consumers before refactoring | `fallow dead-code --type-aware --symbol-impact <file>:<export-or-class.method>` |
| find how one module reaches another | `fallow trace --path <from> <to>` |
| delete an "unused" dependency | `fallow dead-code --trace-dependency <name>` |
| commit or open a PR | `fallow audit --base <ref>` |
| prioritize refactoring | `fallow health --hotspots --targets` |
| consolidate duplication | `fallow dupes --trace dup:<fingerprint>` |
| check which architecture rules apply to a file before changing it | `fallow guard <files>` |
| surface security candidates | `fallow security` |
| understand a finding | `fallow explain <issue-type>` |
<!-- generated:task-matrix:end -->
