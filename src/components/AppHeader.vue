<template>
  <header class="shadow">
    <div class="container px-4 py-4 flex items-center">
      <div class="flex-grow">
        <h1 class="text-2xl font-bold">Knowling</h1>
      </div>
      <nav>
        <ul class="flex space-x-4">
          <li>
            <RouterLink :to="{name: 'Home'}" class="">Notes</RouterLink>
          </li>
          <li>
            <button @click="navigateToNewNote" class="">+</button>
          </li>
          <li>
            <RouterLink :to="{name: 'Settings'}" class="">
              <img src="/settings.svg" alt="Settings" class="icon"/>
            </RouterLink>
          </li>

        </ul>
      </nav>
    </div>
  </header>
</template>

<script setup>
import {RouterLink, useRouter} from 'vue-router'

const router = useRouter()

/**
 * ## Make all NewNote routes unique
 * Navigate to the NewNote route with a timestamp as a parameter
 * This is needed (in addition to using :key (<router-view :key="route.path">)
 * in App.vue to convince Vue to re-render Editor.vue when in the case we are
 * already on Editor.vue and we navigate to a new note.
 */
function navigateToNewNote() {
  const timestamp = Date.now().toString()
  router.push({name: 'NewNote', params: {new: timestamp}})
}

</script>