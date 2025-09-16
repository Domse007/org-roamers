<script setup lang="ts">
import { THEMES } from "../../theme_list.ts";
import ThemeEntry from "./ThemeEntry.vue";
</script>

<template>
  <div class="settings-theme">
    <section class="settings-section">
      <h3 class="section-title">Available Themes</h3>
      <p class="section-description">
        Choose a color theme for the interface. Changes apply immediately.
      </p>
      <div class="theme-grid">
        <ThemeEntry
          v-for="theme in THEMES"
          :key="theme.name"
          :theme="theme"
          @redraw-graph="$emit('redrawGraph')"
        />
      </div>
    </section>
  </div>
</template>

<style scoped>
.settings-theme {
  padding: 16px;
  height: 100%;
  overflow-y: auto;
  box-sizing: border-box;
}

.settings-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--highlight);
  margin: 0 0 8px 0;
  padding-bottom: 4px;
  border-bottom: 1px solid color-mix(in srgb, var(--highlight) 20%, transparent);
}

.section-description {
  font-size: 13px;
  color: var(--overlay);
  margin: 0 0 16px 0;
  line-height: 1.4;
}

.theme-grid {
  display: grid;
  gap: 8px;
  grid-template-columns: 1fr;
}

/* Responsive adjustments */
@media (max-width: 600px) {
  .settings-theme {
    padding: 12px;
  }

  .section-title {
    font-size: 14px;
  }

  .section-description {
    font-size: 12px;
  }
}
</style>
