import { createApp } from "vue";
import "@unocss/reset/tailwind.css";
import "virtual:uno.css";
import "vue-sonner/style.css";
import "../shared/styles/theme.css";
import InspectorApp from "./InspectorApp.vue";

createApp(InspectorApp).mount("#app");
