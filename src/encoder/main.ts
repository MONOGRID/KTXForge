import { createApp } from "vue";
import "@unocss/reset/tailwind.css";
import "virtual:uno.css";
import "../shared/styles/theme.css";
import EncoderApp from "./EncoderApp.vue";

createApp(EncoderApp).mount("#app");
