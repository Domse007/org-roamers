import "./assets/main.css";
import "katex/dist/katex.min.css";

import { createApp } from "vue";
import App from "./App.vue";
import { initializeTheme } from "./theme.ts";

// Initialize theme from localStorage before mounting the app
initializeTheme();

createApp(App).mount("#app");
