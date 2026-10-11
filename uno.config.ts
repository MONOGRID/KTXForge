import {
  defineConfig,
  presetAttributify,
  presetIcons,
  presetWind3,
  transformerDirectives,
  transformerVariantGroup,
} from "unocss";
import presetAnimations from "unocss-preset-animations";
import presetShadcn from "unocss-preset-shadcn/v3";

export default defineConfig({
  presets: [
    presetWind3(),
    presetAttributify(),
    presetIcons(),
    presetAnimations(),
    presetShadcn(
      {
        // Palette overridden in src/shared/styles/theme.css (VS Code–inspired).
        color: "neutral",
        darkSelector: ".dark",
      },
      {
        componentLibrary: "reka",
      },
    ),
  ],
  transformers: [transformerDirectives(), transformerVariantGroup()],
  content: {
    pipeline: {
      include: [
        /\.(vue|svelte|[jt]sx|mdx?|astro|elm|php|phtml|html)($|\?)/,
        "(components|src)/**/*.{js,ts}",
      ],
    },
  },
  theme: {
    colors: {
      surface: {
        DEFAULT: "var(--color-surface)",
        raised: "var(--color-surface-raised)",
        sunken: "var(--color-surface-sunken)",
      },
      ink: {
        DEFAULT: "var(--color-ink)",
        muted: "var(--color-ink-muted)",
        faint: "var(--color-ink-faint)",
      },
      line: "var(--color-line)",
      brand: {
        DEFAULT: "var(--color-brand)",
        fg: "var(--color-brand-fg)",
      },
      danger: {
        DEFAULT: "var(--color-danger)",
        fg: "var(--color-danger-fg)",
      },
    },
    fontFamily: {
      sans: "var(--font-sans)",
      mono: "var(--font-mono)",
    },
  },
  shortcuts: {
    btn: "inline-flex items-center justify-center gap-1.5 rounded border border-line bg-surface-raised px-2.5 py-1 text-sm text-ink hover:bg-surface disabled:opacity-50 disabled:pointer-events-none",
    "btn-primary":
      "btn border-brand bg-brand text-brand-fg hover:opacity-90",
    panel: "rounded border border-line bg-surface-raised text-ink",
    "field-row":
      "grid grid-cols-[minmax(0,42%)_minmax(0,1fr)] items-start gap-x-2 py-1.5",
    label:
      "min-w-0 cursor-pointer truncate pt-1 text-sm text-ink-muted select-none",
    hint: "mt-0.5 text-xs text-ink-faint",
    badge:
      "inline-flex items-center rounded px-1.5 py-0.5 text-xs font-medium bg-surface-sunken text-ink-muted",
  },
});
