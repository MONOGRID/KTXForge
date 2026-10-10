import {
  defineConfig,
  presetAttributify,
  presetIcons,
  presetWind3,
  transformerDirectives,
  transformerVariantGroup,
} from "unocss";

export default defineConfig({
  presets: [presetWind3(), presetAttributify(), presetIcons()],
  transformers: [transformerDirectives(), transformerVariantGroup()],
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
      accent: {
        DEFAULT: "var(--color-accent)",
        fg: "var(--color-accent-fg)",
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
      "btn border-accent bg-accent text-accent-fg hover:opacity-90",
    panel:
      "rounded border border-line bg-surface-raised text-ink",
    "field-row": "flex items-start gap-3 py-1.5",
    label: "min-w-28 shrink-0 pt-1 text-sm text-ink-muted",
    hint: "mt-0.5 text-xs text-ink-faint",
    badge:
      "inline-flex items-center rounded px-1.5 py-0.5 text-xs font-medium bg-surface-sunken text-ink-muted",
  },
});
