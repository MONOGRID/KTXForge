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
        // Palette overridden in src/shared/styles/theme.css (logo blue, soft chrome).
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
        // Relative color so bg-brand/10 etc. tint instead of solid hex.
        DEFAULT: "rgb(from var(--color-brand) r g b / <alpha-value>)",
        fg: "rgb(from var(--color-brand-fg) r g b / <alpha-value>)",
      },
      danger: {
        DEFAULT: "rgb(from var(--color-danger) r g b / <alpha-value>)",
        fg: "rgb(from var(--color-danger-fg) r g b / <alpha-value>)",
      },
    },
    borderRadius: {
      lg: "var(--radius)",
      md: "calc(var(--radius) - 2px)",
      sm: "calc(var(--radius) - 4px)",
    },
    fontFamily: {
      sans: "var(--font-sans)",
      mono: "var(--font-mono)",
    },
  },
  shortcuts: {
    btn: "inline-flex items-center justify-center gap-1.5 rounded-md border border-transparent bg-surface-sunken px-2.5 py-1 text-sm text-ink shadow-sm hover:bg-surface disabled:opacity-50 disabled:pointer-events-none",
    "btn-primary":
      "btn border-transparent bg-brand text-brand-fg shadow-sm hover:opacity-90",
    panel: "rounded-lg border border-transparent bg-surface-raised text-ink shadow-sm",
    "field-row":
      "grid grid-cols-[minmax(0,42%)_minmax(0,1fr)] items-start gap-x-2 py-0.5",
    label:
      "min-w-0 cursor-pointer truncate pt-0.5 text-xs text-ink-muted select-none",
    hint: "mt-0.5 text-xs text-ink-faint",
    badge:
      "inline-flex items-center rounded-md px-1.5 py-0.5 text-xs font-medium bg-surface-sunken text-ink-muted",
  },
});
