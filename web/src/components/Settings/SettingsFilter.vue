<script setup lang="ts"></script>

<template>
  <div class="settings-filter">
    <section class="settings-section">
      <h3 class="section-title">Graph Filters</h3>
      <p class="section-description">
        Control which nodes and connections are displayed in the graph.
      </p>

      <div class="filter-placeholder">
        <div class="placeholder-icon">🔧</div>
        <h4 class="placeholder-title">Filter Settings</h4>
        <p class="placeholder-text">
          Graph filtering options will be available here in future updates. This
          might include filtering by file type, node connections, or custom
          tags.
        </p>
      </div>
    </section>
  </div>
</template>

<style scoped>
.settings-filter {
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
  margin: 0 0 20px 0;
  line-height: 1.4;
}

.filter-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px 16px;
  text-align: center;
  background: color-mix(in srgb, var(--base) 50%, transparent);
  border: 1px dashed color-mix(in srgb, var(--overlay) 40%, transparent);
  border-radius: 4px;
}

.placeholder-icon {
  font-size: 32px;
  margin-bottom: 12px;
  opacity: 0.6;
}

.placeholder-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text);
  margin: 0 0 8px 0;
}

.placeholder-text {
  font-size: 13px;
  color: var(--overlay);
  line-height: 1.5;
  max-width: 300px;
  margin: 0;
}

/* Responsive adjustments */
@media (max-width: 600px) {
  .settings-filter {
    padding: 12px;
  }

  .section-title {
    font-size: 14px;
  }

  .section-description {
    font-size: 12px;
  }

  .filter-placeholder {
    padding: 24px 12px;
  }

  .placeholder-icon {
    font-size: 28px;
  }

  .placeholder-title {
    font-size: 15px;
  }

  .placeholder-text {
    font-size: 12px;
  }
}
</style>
