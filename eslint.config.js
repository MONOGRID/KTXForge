import eslint from "@eslint/js";
import eslintPluginVue from "eslint-plugin-vue";
import globals from "globals";
import typescriptEslint from "typescript-eslint";

export default typescriptEslint.config(
  { ignores: ["**/*.d.ts", "**/coverage", "**/dist", "src-tauri/**"] },
  {
    extends: [
      eslint.configs.recommended,
      ...typescriptEslint.configs.strictTypeChecked,
      ...eslintPluginVue.configs["flat/recommended"],
    ],
    files: ["**/*.{ts,vue}"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: globals.browser,
      parserOptions: {
        parser: typescriptEslint.parser,
        projectService: true,
        extraFileExtensions: [".vue"],
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      "vue/multi-word-component-names": "off",
    },
  },
);
