<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const greetMsg = ref("");
const name = ref("");

async function greet() {
  greetMsg.value = await invoke<string>("greet", { name: name.value });
}
</script>

<template>
  <main class="container">
    <h1>紧固助手（个人版）</h1>
    <p class="subtitle">Tauri v2 + Vue 3 + Vite + TypeScript 项目骨架</p>

    <form class="row" @submit.prevent="greet">
      <input v-model="name" placeholder="输入名称..." />
      <button type="submit">发送</button>
    </form>

    <p class="result">{{ greetMsg }}</p>
  </main>
</template>

<style scoped>
.container {
  margin: 0;
  padding: 2rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  font-family: system-ui, -apple-system, "PingFang SC", "Microsoft YaHei", sans-serif;
}

.subtitle {
  color: #6b7280;
  font-size: 0.9rem;
}

.row {
  display: flex;
  gap: 0.5rem;
}

input {
  padding: 0.5rem 0.75rem;
  border: 1px solid #d1d5db;
  border-radius: 6px;
}

button {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 6px;
  background: #2563eb;
  color: #fff;
  cursor: pointer;
}

.result {
  min-height: 1.5rem;
  color: #111827;
}
</style>
