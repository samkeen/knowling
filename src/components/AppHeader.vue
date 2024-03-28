<template>
  <header class="bg-white shadow">
    <div class="container mx-auto px-4 py-4 flex items-center">
      <div class="flex-grow">
        <h1 class="text-2xl font-bold">My Notes App</h1>
      </div>
      <nav>
        <ul class="flex space-x-4">
          <li>
            <RouterLink :to="{name: 'Home'}" class="">Notes</RouterLink>
          </li>
          <li>
            <RouterLink :to="{name: 'NewNote'}" class="">+</RouterLink>
          </li>
          <li>
            <RouterLink :to="{name: 'MdTest'}" class="">MD</RouterLink>
          </li>

        </ul>
      </nav>
    </div>
  </header>
</template>
<script setup>
import {RouterLink} from 'vue-router'
</script>