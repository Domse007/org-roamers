<script setup lang="ts">
import { ref } from "vue";
import { useAuth } from "../composables/useAuth";

const { login, isLoading, error } = useAuth();

const username = ref("");
const password = ref("");

const emit = defineEmits<{
  loginSuccess: [];
}>();

const handleSubmit = async () => {
  if (!username.value || !password.value) {
    return;
  }

  const success = await login({
    username: username.value,
    password: password.value,
  });

  if (success) {
    emit("loginSuccess");
  }
};

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === "Enter") {
    handleSubmit();
  }
};
</script>

<template>
  <div class="login-container">
    <div class="login-box">
      <h1>org-roamers Login</h1>
      <form @submit.prevent="handleSubmit">
        <div class="form-group">
          <label for="username">Username</label>
          <input
            id="username"
            v-model="username"
            type="text"
            placeholder="Enter username"
            :disabled="isLoading"
            @keydown="handleKeydown"
            autocomplete="username"
          />
        </div>

        <div class="form-group">
          <label for="password">Password</label>
          <input
            id="password"
            v-model="password"
            type="password"
            placeholder="Enter password"
            :disabled="isLoading"
            @keydown="handleKeydown"
            autocomplete="current-password"
          />
        </div>

        <div v-if="error" class="error-message">
          {{ error }}
        </div>

        <button type="submit" :disabled="isLoading || !username || !password">
          {{ isLoading ? "Logging in..." : "Login" }}
        </button>
      </form>
    </div>
  </div>
</template>

<style scoped>
.login-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background: linear-gradient(
    135deg,
    var(--base),
    color-mix(in srgb, var(--base) 97%, var(--surface))
  );
  font-family: var(--font);
}

.login-box {
  background: var(--surface);
  padding: 2rem;
  border-radius: 10px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.3);
  width: 100%;
  max-width: 400px;
  border: 1px solid var(--overlay);
}

h1 {
  margin-top: 0;
  margin-bottom: 1.5rem;
  text-align: center;
  color: var(--text);
  font-size: 1.75rem;
}

.form-group {
  margin-bottom: 1.25rem;
}

label {
  display: block;
  margin-bottom: 0.5rem;
  color: var(--highlight);
  font-weight: 500;
}

input {
  width: 100%;
  padding: 0.75rem;
  background: var(--base);
  border: 2px solid var(--overlay);
  border-radius: 5px;
  font-size: 1rem;
  color: var(--text);
  transition: border-color 0.3s;
  box-sizing: border-box;
}

input:focus {
  outline: none;
  border-color: var(--clickable);
}

input:disabled {
  background-color: color-mix(in srgb, var(--base) 80%, var(--overlay));
  cursor: not-allowed;
  opacity: 0.6;
}

button {
  width: 100%;
  padding: 0.75rem;
  background: var(--clickable);
  color: var(--base);
  border: none;
  border-radius: 5px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.3s;
}

button:hover:not(:disabled) {
  opacity: 0.9;
}

button:active:not(:disabled) {
  opacity: 0.8;
}

button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.error-message {
  background-color: color-mix(in srgb, var(--warn) 20%, var(--surface));
  color: var(--warn);
  padding: 0.75rem;
  border-radius: 5px;
  border: 1px solid var(--warn);
  margin-bottom: 1rem;
  text-align: center;
  font-size: 0.9rem;
}
</style>
