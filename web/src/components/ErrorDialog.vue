<script setup lang="ts">
defineEmits(["dialogClose"]);
</script>

<template>
  <div class="error-dialog">
    <div class="error-header">
      <div class="error-icon">⚠️</div>
      <div class="error-title">Error</div>
      <button
        class="error-close-button"
        @click="$emit('dialogClose')"
        aria-label="Close error dialog"
      >
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
          <path
            d="M1 1l12 12M13 1L1 13"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          />
        </svg>
      </button>
    </div>
    <div class="error-content">
      <div class="error-message"><slot /></div>
    </div>
  </div>
</template>

<style scoped>
.error-dialog {
  position: absolute;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  width: min(400px, 90vw);
  z-index: 150;
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--warn) 15%, var(--surface)),
    color-mix(in srgb, var(--warn) 8%, var(--surface))
  );
  border: 1px solid color-mix(in srgb, var(--warn) 30%, transparent);
  border-radius: 12px;
  box-shadow:
    0 4px 20px rgba(0, 0, 0, 0.15),
    0 1px 3px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
  font-family: var(--font);
  backdrop-filter: blur(10px);
  animation: errorSlideIn 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes errorSlideIn {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(-20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0) scale(1);
  }
}

.error-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px 20px 12px 20px;
  border-bottom: 1px solid color-mix(in srgb, var(--warn) 20%, transparent);
}

.error-icon {
  font-size: 18px;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.3));
}

.error-title {
  flex: 1;
  font-size: 16px;
  font-weight: 600;
  color: var(--warn);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  letter-spacing: 0.5px;
}

.error-close-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: color-mix(in srgb, var(--surface) 80%, transparent);
  color: var(--text);
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.2, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.error-close-button:before {
  content: "";
  position: absolute;
  inset: 0;
  background: radial-gradient(
    circle at center,
    color-mix(in srgb, var(--text) 15%, transparent) 0%,
    transparent 70%
  );
  opacity: 0;
  transition: opacity 0.2s ease;
}

.error-close-button:hover {
  background: color-mix(in srgb, var(--text) 10%, var(--surface));
  transform: scale(1.05);
}

.error-close-button:hover:before {
  opacity: 1;
}

.error-close-button:active {
  transform: scale(0.95);
  transition-duration: 0.1s;
}

.error-close-button svg {
  position: relative;
  z-index: 1;
  transition: transform 0.2s ease;
}

.error-close-button:hover svg {
  transform: rotate(90deg);
}

.error-content {
  padding: 12px 20px 20px 20px;
}

.error-message {
  color: var(--text);
  line-height: 1.5;
  font-size: 14px;
  margin: 0;
  word-wrap: break-word;
}

/* Responsive design */
@media (max-width: 768px) {
  .error-dialog {
    top: 10px;
    width: min(350px, 95vw);
  }

  .error-header {
    padding: 14px 16px 10px 16px;
  }

  .error-content {
    padding: 10px 16px 16px 16px;
  }

  .error-title {
    font-size: 15px;
  }

  .error-message {
    font-size: 13px;
  }
}

/* Dark theme enhancements */
@media (prefers-color-scheme: dark) {
  .error-dialog {
    box-shadow:
      0 4px 20px rgba(0, 0, 0, 0.3),
      0 1px 3px rgba(0, 0, 0, 0.2),
      inset 0 1px 0 rgba(255, 255, 255, 0.05);
  }
}

/* Reduced motion preference */
@media (prefers-reduced-motion: reduce) {
  .error-dialog {
    animation: none;
  }

  .error-close-button {
    transition: none;
  }

  .error-close-button svg {
    transition: none;
  }

  .error-close-button:hover svg {
    transform: none;
  }
}
</style>
