import { ref, onUnmounted } from "vue";

/**
 * Composable for handling preview frame resizing functionality
 */
export function usePreviewResize() {
  const frameWidth = ref(50); // Default width as percentage
  const isResizing = ref(false);

  const startResize = (event: MouseEvent) => {
    isResizing.value = true;
    document.addEventListener("mousemove", doResize);
    document.addEventListener("mouseup", stopResize);
    document.body.style.cursor = "ew-resize";
    document.body.style.userSelect = "none"; // Prevent text selection during resize
    event.preventDefault();
  };

  const doResize = (event: MouseEvent) => {
    if (!isResizing.value) return;

    const windowWidth = window.innerWidth;
    const newWidth = ((windowWidth - event.clientX) / windowWidth) * 100;

    // Constrain width between 20% and 80%
    frameWidth.value = Math.max(20, Math.min(80, newWidth));
  };

  const stopResize = () => {
    isResizing.value = false;
    document.removeEventListener("mousemove", doResize);
    document.removeEventListener("mouseup", stopResize);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
  };

  // Cleanup event listeners on component unmount
  onUnmounted(() => {
    if (isResizing.value) {
      stopResize();
    }
  });

  return {
    frameWidth,
    isResizing,
    startResize,
    stopResize,
  };
}
