import { ref, type Ref } from "vue";

export interface RouterState {
  nodeId: string;
  timestamp: number;
}

/**
 * Router composable that synchronizes navigation with browser history
 *
 * This provides a Vue-friendly wrapper around the browser's History API
 * and coordinates navigation state across the application.
 */
export function useRouter() {
  const currentNodeId: Ref<string> = ref("");
  const isNavigating = ref(false);

  // Callback registry for navigation events
  const navigationCallbacks: Array<(nodeId: string) => void> = [];

  /**
   * Initialize the router and set up browser history listeners
   */
  const initialize = () => {
    // Handle browser back/forward buttons
    window.addEventListener("popstate", (event) => {
      console.log("Browser navigation (popstate):", event.state);

      if (event.state && event.state.nodeId) {
        isNavigating.value = true;
        const nodeId = event.state.nodeId;
        currentNodeId.value = nodeId;

        // Notify all registered callbacks
        navigationCallbacks.forEach((callback) => callback(nodeId));

        isNavigating.value = false;
      }
    });

    // Read initial state from URL
    const urlParams = new URLSearchParams(window.location.search);
    const initialNodeId = urlParams.get("node");

    if (initialNodeId) {
      console.log("Initial node from URL:", initialNodeId);
      currentNodeId.value = initialNodeId;

      // Replace the initial state to make back button work
      const state: RouterState = {
        nodeId: initialNodeId,
        timestamp: Date.now(),
      };
      window.history.replaceState(
        state,
        "",
        `?node=${encodeURIComponent(initialNodeId)}`,
      );
    }
  };

  /**
   * Navigate to a new node
   * This will update the browser history and URL
   */
  const push = (nodeId: string) => {
    if (!nodeId) {
      console.warn("Router.push: Cannot navigate to empty node ID");
      return;
    }

    if (isNavigating.value) {
      // Prevent recursive navigation during popstate handling
      console.log("Router.push: Navigation already in progress, skipping");
      return;
    }

    if (nodeId === currentNodeId.value) {
      console.log("Router.push: Already on node:", nodeId);
      return;
    }

    console.log("Router push:", nodeId);
    currentNodeId.value = nodeId;

    const state: RouterState = {
      nodeId,
      timestamp: Date.now(),
    };

    // Update browser history and URL
    window.history.pushState(state, "", `?node=${encodeURIComponent(nodeId)}`);
  };

  /**
   * Replace the current history entry without adding a new one
   * Useful for initial navigation or corrections
   */
  const replace = (nodeId: string) => {
    if (!nodeId) {
      console.warn("Router.replace: Cannot navigate to empty node ID");
      return;
    }

    console.log("Router replace:", nodeId);
    currentNodeId.value = nodeId;

    const state: RouterState = {
      nodeId,
      timestamp: Date.now(),
    };

    window.history.replaceState(
      state,
      "",
      `?node=${encodeURIComponent(nodeId)}`,
    );
  };

  /**
   * Register a callback to be called when navigation occurs
   * Returns a function to unregister the callback
   */
  const onNavigate = (callback: (nodeId: string) => void): (() => void) => {
    navigationCallbacks.push(callback);

    // Return unregister function
    return () => {
      const index = navigationCallbacks.indexOf(callback);
      if (index > -1) {
        navigationCallbacks.splice(index, 1);
      }
    };
  };

  /**
   * Get the current node ID from the router
   */
  const getCurrentNodeId = (): string => {
    return currentNodeId.value;
  };

  return {
    // State
    currentNodeId,

    // Methods
    initialize,
    push,
    replace,
    onNavigate,
    getCurrentNodeId,
  };
}

// Create a singleton instance for the app
let routerInstance: ReturnType<typeof useRouter> | null = null;

/**
 * Get the router instance (creates one if it doesn't exist)
 * This ensures all components use the same router
 */
export function getRouter(): ReturnType<typeof useRouter> {
  if (!routerInstance) {
    routerInstance = useRouter();
  }
  return routerInstance;
}
