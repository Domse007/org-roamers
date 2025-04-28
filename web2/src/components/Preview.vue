<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{ id: string }>();

// TODO: everything form here on down must be ported to vue code.
const rendered = ref("");
const dividerPosition = ref(0);

const preview = (id: string) => {
  console.log(`Previewing ${id}`);
  fetch(`/org?title=${id}`)
    .then((response) => {
      return response.text();
    })
    .then((html) => {
      rendered.value = html;
    });
};

watch(props, () => preview(props.id));
</script>

<template>
  <div id="org-preview-frame">
    <div id="org-preview" v-html="rendered"></div>
  </div>
</template>

<style>
#org-preview-frame {
  background-color: var(--surface);
  border-left: 5px solid var(--clickable);
  color: white;
  width: 40%;
  right: 0px;
  top: 0px;
  bottom: 0px;
  position: absolute;
  z-index: 50;
  resize: horizontal;
  float: left;
  overflow: scroll;
  z-index: 50;
}

#org-preview {
  background-color: var(--surface);
  font-family: var(--font);
  padding: 10px;
  width: 100%;
}

h1,
h2,
h3,
h4 {
  color: var(--highlight-2);
}

hr {
  color: var(--highlight);
}

.center {
  text-align: center;
}
</style>
