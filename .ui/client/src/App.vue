<template>
  <v-app>
    <!-- App Bar with Social Links -->
    <v-app-bar color="primary">
      <v-toolbar-title class="text-h4">
        <span class="font-weight-bold">Stack</span>
        <span class="font-weight-light">Vo</span>
      </v-toolbar-title>

      <v-spacer></v-spacer>

      <!-- Primary Links -->
      <v-btn icon href="https://stackvo.github.io/stackvo" target="_blank" title="Documentation">
        <v-icon>mdi-book-open-variant</v-icon>
      </v-btn>

      <v-btn icon href="https://github.com/stackvo/stackvo" target="_blank" title="GitHub">
        <v-icon>mdi-github</v-icon>
      </v-btn>

      <v-btn icon href="https://buymeacoffee.com/fahrettinaksoy" target="_blank" title="Buy Me a Coffee">
        <v-icon>mdi-coffee</v-icon>
      </v-btn>

      <!-- Social Media Dropdown -->
      <v-menu>
        <template v-slot:activator="{ props }">
          <v-btn icon v-bind="props" title="Social Media">
            <v-icon>mdi-share-variant</v-icon>
          </v-btn>
        </template>
        <v-list>
          <v-list-item href="https://www.youtube.com/stackvo" target="_blank">
            <template v-slot:prepend>
              <v-icon>mdi-youtube</v-icon>
            </template>
            <v-list-item-title>YouTube</v-list-item-title>
          </v-list-item>

          <v-list-item href="https://fosstodon.org/@stackvo" target="_blank">
            <template v-slot:prepend>
              <v-icon>mdi-mastodon</v-icon>
            </template>
            <v-list-item-title>Mastodon</v-list-item-title>
          </v-list-item>

          <v-list-item href="https://www.linkedin.com/company/stackvo" target="_blank">
            <template v-slot:prepend>
              <v-icon>mdi-linkedin</v-icon>
            </template>
            <v-list-item-title>LinkedIn</v-list-item-title>
          </v-list-item>

          <v-list-item href="https://reddit.com/r/stackvo" target="_blank">
            <template v-slot:prepend>
              <v-icon>mdi-reddit</v-icon>
            </template>
            <v-list-item-title>Reddit</v-list-item-title>
          </v-list-item>

          <v-list-item href="https://bsky.app/profile/stackvo" target="_blank">
            <template v-slot:prepend>
              <v-icon>mdi-cloud</v-icon>
            </template>
            <v-list-item-title>Bluesky</v-list-item-title>
          </v-list-item>

          <v-list-item href="https://twitter.com/stackvo" target="_blank">
            <template v-slot:prepend>
              <v-icon>mdi-twitter</v-icon>
            </template>
            <v-list-item-title>Twitter/X</v-list-item-title>
          </v-list-item>

          <v-list-item href="https://discord.gg/stackvo" target="_blank">
            <template v-slot:prepend>
              <v-icon>mdi-discord</v-icon>
            </template>
            <v-list-item-title>Discord</v-list-item-title>
          </v-list-item>
        </v-list>
      </v-menu>

      <v-divider vertical class="mx-2"></v-divider>

      <!-- Theme Toggle -->
      <v-btn icon @click="toggleTheme" title="Toggle Theme">
        <v-icon>{{ theme.global.current.value.dark ? 'mdi-weather-sunny' : 'mdi-weather-night' }}</v-icon>
      </v-btn>
    </v-app-bar>

    <!-- Left Navigation Drawer -->
    <v-navigation-drawer location="left" permanent width="190" class="elevation-1 border-0">
      <!-- Top Section: Navigation and System Status -->
      <v-list nav density="compact">
        <v-list-subheader>NAVIGATION</v-list-subheader>

        <v-list-item 
          prepend-icon="mdi-view-dashboard" 
          title="Dashboard" 
          :active="currentTab === 'dashboard'"
          @click="navigateTo('dashboard')"
        ></v-list-item>

        <v-list-item 
          prepend-icon="mdi-folder-multiple" 
          title="Projects" 
          :active="currentTab === 'projects'"
          @click="navigateTo('projects')"
        ></v-list-item>

        <v-list-item 
          prepend-icon="mdi-server" 
          title="Services" 
          :active="currentTab === 'services'"
          @click="navigateTo('services')"
        ></v-list-item>

        <v-list-item 
          prepend-icon="mdi-tools" 
          title="Tools" 
          :active="currentTab === 'tools'"
          @click="navigateTo('tools')"
        ></v-list-item>

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

      <!-- Spacer to push Quick Actions to bottom -->
      <v-spacer></v-spacer>

      <!-- Bottom Section: Quick Actions -->
      <v-list nav density="compact" class="pb-2">
        <v-divider class="mb-2"></v-divider>
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
      </v-list>
    </v-navigation-drawer>

    <!-- Right Navigation Drawer -->
     <!-- 
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
    -->

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

/* Left Navigation Drawer - Flex Column Layout */
:deep(.v-navigation-drawer__content) {
  display: flex;
  flex-direction: column;
}
</style>
