<script setup lang="ts">
import { greet } from "./greet";

const props = defineProps<{ name: string }>();
</script>

<template>
  <p>{{ greet(props.name) }}</p>
</template>
