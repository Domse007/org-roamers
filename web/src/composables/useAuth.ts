import { ref, type Ref } from "vue";

export interface LoginCredentials {
  username: string;
  password: string;
}

export interface SessionInfo {
  authenticated: boolean;
  username: string | null;
}

/**
 * Composable for handling authentication
 */
export function useAuth() {
  const isAuthenticated: Ref<boolean> = ref(false);
  const username: Ref<string | null> = ref(null);
  const isLoading: Ref<boolean> = ref(false);
  const error: Ref<string | null> = ref(null);

  /**
   * Login with username and password
   */
  const login = async (credentials: LoginCredentials): Promise<boolean> => {
    isLoading.value = true;
    error.value = null;

    try {
      const response = await fetch("/api/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include", // Important for cookies!
        body: JSON.stringify(credentials),
      });

      if (response.ok) {
        const data = await response.json();
        isAuthenticated.value = true;
        username.value = data.username;
        console.log("Login successful:", data.username);
        return true;
      } else {
        error.value =
          response.status === 401
            ? "Invalid username or password"
            : "Login failed";
        return false;
      }
    } catch (err) {
      console.error("Login error:", err);
      error.value = "Network error. Please check your connection.";
      return false;
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Logout current user
   */
  const logout = async (): Promise<void> => {
    try {
      await fetch("/api/logout", {
        method: "POST",
        credentials: "include",
      });
    } catch (err) {
      console.error("Logout error:", err);
    } finally {
      isAuthenticated.value = false;
      username.value = null;
    }
  };

  /**
   * Check if user has an active session
   */
  const checkSession = async (): Promise<boolean> => {
    try {
      const response = await fetch("/api/session", {
        credentials: "include",
      });

      if (response.ok) {
        const data: SessionInfo = await response.json();
        isAuthenticated.value = data.authenticated;
        username.value = data.username;
        console.log("Session check:", data);
        return data.authenticated;
      } else {
        isAuthenticated.value = false;
        username.value = null;
        return false;
      }
    } catch (err) {
      console.error("Session check error:", err);
      isAuthenticated.value = false;
      username.value = null;
      return false;
    }
  };

  return {
    // State
    isAuthenticated,
    username,
    isLoading,
    error,

    // Methods
    login,
    logout,
    checkSession,
  };
}
