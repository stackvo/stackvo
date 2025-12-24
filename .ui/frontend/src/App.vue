<template>
  <v-app>
    <!-- App Bar with Tabs -->
    <v-app-bar color="primary" prominent>
      <v-toolbar-title>
        <v-icon icon="mdi-docker" size="large" class="mr-2"></v-icon>
        Stackvo
      </v-toolbar-title>

      <v-tabs v-model="currentView" color="white">
        <v-tab value="dashboard">
          <v-icon start>mdi-view-dashboard</v-icon>
          Dashboard
        </v-tab>
        <v-tab value="projects">
          <v-icon start>mdi-folder-multiple</v-icon>
          Projects
        </v-tab>
        <v-tab value="services">
          <v-icon start>mdi-server</v-icon>
          Services
        </v-tab>
        <v-tab value="tools">
          <v-icon start>mdi-tools</v-icon>
          Tools
        </v-tab>
      </v-tabs>

      <v-btn icon @click="toggleTheme">
        <v-icon>{{ theme.global.current.value.dark ? 'mdi-weather-sunny' : 'mdi-weather-night' }}</v-icon>
      </v-btn>
    </v-app-bar>

    <!-- Right Navigation Drawer -->
    <v-navigation-drawer location="right" permanent width="280" class="elevation-1 border-0">
      <v-list nav density="compact">
        <v-list-subheader>QUICK ACTIONS</v-list-subheader>

        <v-list-item prepend-icon="mdi-play-circle" title="Start All" subtitle="stackvo up" :disabled="commandLoading">
          <template v-slot:append v-if="commandLoading">
            <v-progress-circular indeterminate size="20"></v-progress-circular>
          </template>
        </v-list-item>

        <v-list-item prepend-icon="mdi-stop-circle" title="Stop All" subtitle="stackvo down" :disabled="commandLoading">
          <template v-slot:append v-if="commandLoading">
            <v-progress-circular indeterminate size="20"></v-progress-circular>
          </template>
        </v-list-item>

        <v-list-item prepend-icon="mdi-restart" title="Restart" subtitle="stackvo restart" :disabled="commandLoading">
          <template v-slot:append v-if="commandLoading">
            <v-progress-circular indeterminate size="20"></v-progress-circular>
          </template>
        </v-list-item>

        <v-divider class="my-2"></v-divider>

        <v-list-subheader>SYSTEM STATUS</v-list-subheader>

        <v-list-item>
          <template v-slot:prepend>
            <v-icon :color="systemStatus.running ? 'success' : 'error'">mdi-circle</v-icon>
          </template>
          <v-list-item-title>Docker</v-list-item-title>
          <v-list-item-subtitle>{{ systemStatus.running ? 'Running' : 'Stopped' }}</v-list-item-subtitle>
        </v-list-item>

        <v-list-item>
          <template v-slot:prepend>
            <v-icon color="info">mdi-memory</v-icon>
          </template>
          <v-list-item-title>Containers</v-list-item-title>
          <v-list-item-subtitle>{{ systemStatus.containerCount }} active</v-list-item-subtitle>
        </v-list-item>
      </v-list>
    </v-navigation-drawer>

    <!-- Main Content -->
    <v-main>
      <v-container fluid>
        <div v-show="currentView === 'dashboard'">
          <Dashboard />
        </div>
        <div v-show="currentView === 'projects'">
          <Projects />
        </div>
        <div v-show="currentView === 'services'">
          <Services />
        </div>
        <div v-show="currentView === 'tools'">
          <Tools />
        </div>
      </v-container>
    </v-main>

    <!-- New Project Drawer -->
    <NewProjectDrawer v-model="newProjectDrawer" @created="handleCreateProject" />
  </v-app>
</template>

<script setup>
import { ref, computed, onMounted, provide } from 'vue';
import { useTheme } from 'vuetify';
import { useServicesStore } from '@/stores/services';
import { useProjectsStore } from '@/stores/projects';
import Dashboard from '@/views/Dashboard.vue';
import Projects from '@/views/Projects.vue';
import Services from '@/views/Services.vue';
import Tools from '@/views/Tools.vue';
import NewProjectDrawer from '@/components/NewProjectDrawer.vue';

const theme = useTheme();
const servicesStore = useServicesStore();
const projectsStore = useProjectsStore();

const currentView = ref('dashboard');
const commandLoading = ref(false);
const newProjectDrawer = ref(false);

// Provide newProjectDrawer to child components
provide('newProjectDrawer', newProjectDrawer);

const systemStatus = computed(() => ({
  running: servicesStore.runningServices.length > 0 || projectsStore.runningProjects.length > 0,
  containerCount: servicesStore.runningServices.length + projectsStore.runningProjects.length
}));

function toggleTheme() {
  theme.global.name.value = theme.global.current.value.dark ? 'light' : 'dark';
  localStorage.setItem('stackvo-theme', theme.global.name.value);
}

async function handleCreateProject(projectData) {
  try {
    await projectsStore.createProject(projectData);
    newProjectDrawer.value = false;
  } catch (error) {
    console.error('Failed to create project:', error);
    alert('Failed to create project: ' + error.message);
  }
}

// Load saved theme
const savedTheme = localStorage.getItem('stackvo-theme') || 'dark';
theme.global.name.value = savedTheme;

// Load initial data
onMounted(async () => {
  await Promise.all([
    servicesStore.loadServices(),
    projectsStore.loadProjects()
  ]);
});
</script>
