<template>
  <v-app>
    <!-- App Bar with Tabs -->
    <v-app-bar color="primary" prominent>
      <v-toolbar-title>
        StackVo
      </v-toolbar-title>

      <v-tabs :model-value="currentTab" color="white" @update:model-value="navigateTo">
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

        <v-list-item 
          prepend-icon="mdi-play-circle" 
          title="Start All" 
          subtitle="stackvo up" 
          :disabled="commandLoading"
          @click="startAll"
        >
          <template v-slot:append v-if="commandLoading">
            <v-progress-circular indeterminate size="20"></v-progress-circular>
          </template>
        </v-list-item>

        <v-list-item 
          prepend-icon="mdi-stop-circle" 
          title="Stop All" 
          subtitle="stackvo down" 
          :disabled="commandLoading"
          @click="stopAll"
        >
          <template v-slot:append v-if="commandLoading">
            <v-progress-circular indeterminate size="20"></v-progress-circular>
          </template>
        </v-list-item>

        <v-list-item 
          prepend-icon="mdi-restart" 
          title="Restart" 
          subtitle="stackvo restart" 
          :disabled="commandLoading"
          @click="restartAll"
        >
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
        <router-view />
      </v-container>
    </v-main>

    <!-- New Project Drawer -->
    <NewProjectDrawer v-model="newProjectDrawer" @created="handleCreateProject" />
  </v-app>
</template>

<script setup>
import { ref, computed, onMounted, provide } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useTheme } from 'vuetify';
import { useServicesStore } from '@/stores/services';
import { useProjectsStore } from '@/stores/projects';
import NewProjectDrawer from '@/components/NewProjectDrawer.vue';

const router = useRouter();
const route = useRoute();
const theme = useTheme();
const servicesStore = useServicesStore();
const projectsStore = useProjectsStore();

const commandLoading = ref(false);
const newProjectDrawer = ref(false);
const isNavigating = ref(false);

// Provide newProjectDrawer to child components
provide('newProjectDrawer', newProjectDrawer);
provide('isNavigating', isNavigating);

// Router navigation guards
router.beforeEach((to, from, next) => {
  isNavigating.value = true;
  next();
});

router.afterEach(() => {
  // Small delay for smooth transition
  setTimeout(() => {
    isNavigating.value = false;
  }, 200);
});

// Compute current tab from route
const currentTab = computed(() => {
  const path = route.path;
  if (path === '/') return 'dashboard';
  if (path.startsWith('/projects')) return 'projects';
  if (path.startsWith('/services')) return 'services';
  if (path.startsWith('/tools')) return 'tools';
  return 'dashboard';
});

// Navigate to route
function navigateTo(tab) {
  const routes = {
    'dashboard': '/',
    'projects': '/projects',
    'services': '/services',
    'tools': '/tools'
  };
  router.push(routes[tab] || '/');
}

const systemStatus = computed(() => ({
  running: servicesStore.runningServices.length > 0 || projectsStore.runningProjects.length > 0,
  containerCount: servicesStore.runningServices.length + projectsStore.runningProjects.length
}));

function toggleTheme() {
  theme.global.name.value = theme.global.current.value.dark ? 'light' : 'dark';
  localStorage.setItem('stackvo-theme', theme.global.name.value);
}

// QUICK ACTIONS
async function startAll() {
  commandLoading.value = true;
  try {
    const response = await fetch('/api/docker/start-all', { method: 'POST' });
    const data = await response.json();
    
    if (data.success) {
      // Reload data
      await Promise.all([
        servicesStore.loadServices(),
        projectsStore.loadProjects()
      ]);
    } else {
      console.error('Start All failed:', data.message);
    }
  } catch (error) {
    console.error('Start All error:', error);
  } finally {
    commandLoading.value = false;
  }
}

async function stopAll() {
  commandLoading.value = true;
  try {
    const response = await fetch('/api/docker/stop-all', { method: 'POST' });
    const data = await response.json();
    
    if (data.success) {
      // Reload data
      await Promise.all([
        servicesStore.loadServices(),
        projectsStore.loadProjects()
      ]);
    } else {
      console.error('Stop All failed:', data.message);
    }
  } catch (error) {
    console.error('Stop All error:', error);
  } finally {
    commandLoading.value = false;
  }
}

async function restartAll() {
  commandLoading.value = true;
  try {
    const response = await fetch('/api/docker/restart-all', { method: 'POST' });
    const data = await response.json();
    
    if (data.success) {
      // Reload data
      await Promise.all([
        servicesStore.loadServices(),
        projectsStore.loadProjects()
      ]);
    } else {
      console.error('Restart All failed:', data.message);
    }
  } catch (error) {
    console.error('Restart All error:', error);
  } finally {
    commandLoading.value = false;
  }
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

<style scoped>
:deep(.v-main) {
  overflow: hidden !important;
  height: calc(100vh - 64px); /* App bar height */
}

:deep(.v-container) {
  overflow: hidden !important;
  height: 100%;
}
</style>
