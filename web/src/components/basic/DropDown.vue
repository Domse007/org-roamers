<script setup lang="ts">
import { onMounted, type Ref, ref } from "vue";

const display: Ref<string> = ref("");
const selected: Ref<boolean> = ref(false);

const props = defineProps<{ items: string[]; initial: number | null }>();

const leadingItem: Ref<HTMLElement | null> = ref(null);
const width: Ref<number> = ref(0);

onMounted(() => {
  display.value =
    props.initial == null ? "Select value" : props.items[props.initial];
  if (leadingItem.value) {
    width.value = leadingItem.value.offsetWidth;
  }
});

const select = (item: string) => {
  selected.value = false;
  display.value = item;
  emit("selected", item);
};

const emit = defineEmits(["selected"]);
</script>

<template>
  <div style="color: var(--text); user-select: none">
    <div
      id="leading"
      class="item"
      style="border: 2px solid var(--clickable)"
      :onclick="() => (selected = true)"
      :style="{
        borderBottomLeftRadius: selected ? '0px' : '5px',
        borderBottomRightRadius: selected ? '0px' : '5px',
      }"
      ref="leadingItem"
    >
      {{ display }}
    </div>
    <div
      v-if="selected"
      style="position: absolute"
      :style="{ width: width + 'px' }"
    >
      <div
        class="dropdown-item item"
        v-for="item in items"
        :onclick="() => select(item)"
      >
        {{ item }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.dropdown-item:hover {
  background-color: var(--highlight);
}

#leading {
  border-top-right-radius: 5px;
  border-top-left-radius: 5px;
}

.item {
  padding: 2px;
  background-color: var(--base);
}
</style>
