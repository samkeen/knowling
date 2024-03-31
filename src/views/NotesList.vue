<template>
  <div class="mb-8 ml-8 mt-2">
    <!--    <h2 class="text-2xl font-bold mb-4">Notes</h2>-->
    <div v-for="(group, groupName) in groupedNotes" :key="groupName">
      <h3 class="text-xl font-bold mb-2">{{ groupName }}</h3>
      <ul>
        <li v-for="note in group" :key="note.id" class="p-2 bg-white">
          <RouterLink :to="{ name: 'EditNote', params: { id: note.id } }">
            <p class="truncate ...">{{ noteTitle(note.text) }}</p>
          </RouterLink>
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup>
import {onMounted, ref, computed} from 'vue';
import {invoke} from "@tauri-apps/api/tauri";
import {RouterLink, createWebHistory} from "vue-router";

const notes = ref([]);

async function get_notes() {
  try {
    let result = await invoke("get_notes");
    console.log("The all notes result: ", result);
    notes.value = result;
  } catch (error) {
    console.error("Failed getting notes:", error);
    // Handle the error as needed, e.g., show a user-friendly message
  }
}

const groupedNotes = computed(() => {
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);

  const groups = {
    Today: [],
    Yesterday: [],
    'Earlier this month': [],
  };

  const monthGroups = {};

  notes.value.forEach(note => {
    const modifiedDate = new Date(note.modified * 1000);

    if (isSameDay(modifiedDate, today)) {
      groups.Today.push(note);
    } else if (isSameDay(modifiedDate, yesterday)) {
      groups.Yesterday.push(note);
    } else if (isSameMonth(modifiedDate, today)) {
      groups['Earlier this month'].push(note);
    } else {
      const monthYear = formatMonthYear(modifiedDate);
      if (!monthGroups[monthYear]) {
        monthGroups[monthYear] = [];
      }
      monthGroups[monthYear].push(note);
    }
  });

  const sortedGroups = {};
  Object.keys(groups).forEach(key => {
    if (groups[key].length > 0) {
      sortedGroups[key] = groups[key];
    }
  });

  const sortedMonthGroups = {};
  Object.keys(monthGroups).sort((a, b) => new Date(b) - new Date(a)).forEach(key => {
    sortedMonthGroups[key] = monthGroups[key];
  });

  return {...sortedGroups, ...sortedMonthGroups};
});

function isSameDay(date1, date2) {
  return (
      date1.getFullYear() === date2.getFullYear() &&
      date1.getMonth() === date2.getMonth() &&
      date1.getDate() === date2.getDate()
  );
}

function isSameMonth(date1, date2) {
  return (
      date1.getFullYear() === date2.getFullYear() &&
      date1.getMonth() === date2.getMonth()
  );
}

function formatMonthYear(date) {
  return date.toLocaleString('default', {month: 'long', year: 'numeric'});
}

function noteTitle(text) {
  const lines = text.split('\n');
  const firstLine = lines[0];
  // remove any leading '#' or spaces
  return firstLine.replace(/^#+\s*/, '');
}

onMounted(get_notes);
</script>