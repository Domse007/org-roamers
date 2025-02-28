import { defineConfig } from "vite";

export default defineConfig(({ command, mode, ssrBuild }) => ({
    build: {
	cssMinify: false,
	minify: false,
    },
}));
