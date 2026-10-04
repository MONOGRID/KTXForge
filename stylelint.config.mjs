/** @type {import('stylelint').Config} */
export default {
  extends: ["stylelint-config-standard-vue/scss"],
  ignoreFiles: ["**/dist/**", "**/src-tauri/**"],
};
