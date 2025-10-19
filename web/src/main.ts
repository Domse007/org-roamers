import "./assets/main.css";
import "katex/dist/katex.min.css";

import { createApp } from "vue";
import App from "./App.vue";
import { initializeTheme } from "./theme.ts";
import { setupSyntaxHighlighting } from "./utils/highlightSetup";

// Initialize theme from localStorage before mounting the app
initializeTheme();

// Initialize syntax highlighting
setupSyntaxHighlighting();

createApp(App).mount("#app");
