<script setup lang="ts">
import { ref, type Ref } from "vue";
import { type SearchResponse } from "../types.ts";
import SearchSuggestion from "./SearchSuggestion.vue";

const searchSuggestions: Ref<
  { display: string; id: string; tags: string[] }[]
> = ref([]);
const searchterm: Ref<string> = ref("");
const showSuggestions: Ref<boolean> = ref(false);

const search = async (query: string) => {
  const encoded = encodeURIComponent(query);
  const resp = await fetch(`/search?q=${encoded}`);
  const text = await resp.json();
  const res = await JSON.parse(text);
  return res;
};

const InputHandler = () => {
  showSuggestions.value = true;
  search(searchterm.value).then((res: SearchResponse) => {
    searchSuggestions.value = res.providers[0].results;
    console.log(searchSuggestions.value);
  });
};

const searchOnLeave = () => {
  setTimeout(() => (showSuggestions.value = false), 100);
};
</script>

<template>
  <div id="search-wrapper">
    <input
      id="search-input"
      placeholder="Search for nodes..."
      type="search"
      @input="InputHandler"
      @focus="InputHandler"
      @blur="searchOnLeave"
      v-model="searchterm"
    />
    <div id="search-suggestion-wrapper">
      <div v-if="showSuggestions">
        <SearchSuggestion
          @open-node="(id) => $emit('openNode', id)"
          v-for="item in searchSuggestions"
          :key="item.id"
          :display="item.display"
          :id="item.id"
          :tags="item.tags"
        >
        </SearchSuggestion>
      </div>
    </div>
  </div>
</template>

<style scoped>
#search-wrapper {
  width: 40%;
  min-width: 400px;
  margin: 5px;
  position: absolute;
  z-index: 100;
  padding: 5px;
  font-family: var(--font);
  background-color: var(--overlay);
  border-radius: 5px;
  transition: top 0.3s ease-in-out;
}

#search-input {
  width: 100%;
  border: 3px solid var(--highlight);
  border-radius: 5px;
  background-color: var(--surface);
  color: var(--text);
  padding: 5px;
}

#search-suggestion-wrapper {
  background-color: var(--surface);
  color: var(--text);
  max-height: 500px;
  overflow-y: scroll;
  overflow-x: hidden;
}
</style>
