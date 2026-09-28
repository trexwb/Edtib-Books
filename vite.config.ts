import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],

  // Tauri 使用固定端口，端口被占用时直接失败而不是静默切换
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 忽略监听 src-tauri，避免 Rust 侧编译产物触发前端热更新
      ignored: ["**/src-tauri/**"],
    },
  },
});
