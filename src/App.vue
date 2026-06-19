<template>
  <AppLayout>
    <router-view />
  </AppLayout>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useAppStore } from '@/store/app'
import AppLayout from '@/components/AppLayout.vue'

const appStore = useAppStore()

onMounted(async () => {
  await appStore.initialize()
})
</script>
