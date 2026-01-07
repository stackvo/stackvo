<template>
  <div class="pa-2">
    <!-- Stats Cards -->
    <v-row>
      <!-- 1. Health - Tüm Containerler -->
      <v-col cols="12" md="3">
        <v-card rounded="0" elevation="1" hover>
          <v-card-text style="min-height: 100px">
            <v-skeleton-loader
              v-if="loading"
              type="paragraph"
              height="88"
            ></v-skeleton-loader>
            <div v-else class="d-flex align-center">
              <v-icon color="info" size="48" class="mr-4"
                >mdi-heart-pulse</v-icon
              >
              <div class="flex-grow-1">
                <div class="text-h4">{{ totalContainers }}</div>
                <div class="text-subtitle-1 text-grey">Health</div>
                <div class="text-caption text-grey">
                  <v-icon size="12" color="success">mdi-circle</v-icon>
                  {{ totalRunning }} Running
                  <v-icon size="12" color="error" class="ml-2"
                    >mdi-circle</v-icon
                  >
                  {{ totalStopped }} Stopped
                </div>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>

      <!-- 2. Projects -->
      <v-col cols="12" md="3">
        <v-card
          rounded="0"
          elevation="1"
          hover
          class="cursor-pointer"
          @click="!loading && $router.push('/projects')"
        >
          <v-card-text style="min-height: 100px">
            <v-skeleton-loader
              v-if="loading"
              type="paragraph"
              height="88"
            ></v-skeleton-loader>
            <div v-else class="d-flex align-center">
              <v-icon color="info" size="48" class="mr-4"
                >mdi-folder-multiple</v-icon
              >
              <div class="flex-grow-1">
                <div class="text-h4">{{ projectsStore.projectsCount }}</div>
                <div class="text-subtitle-1 text-grey">Projects</div>
                <div class="text-caption text-grey">
                  <v-icon size="12" color="success">mdi-circle</v-icon>
                  {{ projectsStore.runningProjects.length }} Active
                  <v-icon size="12" color="grey" class="ml-2"
                    >mdi-circle</v-icon
                  >
                  {{ projectsStore.stoppedProjects.length }} Inactive
                </div>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>

      <!-- 3. Services -->
      <v-col cols="12" md="3">
        <v-card
          rounded="0"
          elevation="1"
          hover
          class="cursor-pointer"
          @click="!loading && $router.push('/services')"
        >
          <v-card-text style="min-height: 100px">
            <v-skeleton-loader
              v-if="loading"
              type="paragraph"
              height="88"
            ></v-skeleton-loader>
            <div v-else class="d-flex align-center">
              <v-icon color="info" size="48" class="mr-4">mdi-server</v-icon>
              <div class="flex-grow-1">
                <div class="text-h4">{{ servicesStore.servicesCount }}</div>
                <div class="text-subtitle-1 text-grey">Services</div>
                <div class="text-caption text-grey">
                  <v-icon size="12" color="success">mdi-circle</v-icon>
                  {{ servicesStore.runningServices.length }} Active
                  <v-icon size="12" color="grey" class="ml-2"
                    >mdi-circle</v-icon
                  >
                  {{ servicesStore.stoppedServices.length }} Inactive
                </div>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>

      <!-- 4. Tools -->
      <v-col cols="12" md="3">
        <v-card
          rounded="0"
          elevation="1"
          hover
          class="cursor-pointer"
          @click="!loading && $router.push('/tools')"
        >
          <v-card-text style="min-height: 100px">
            <v-skeleton-loader
              v-if="loading"
              type="paragraph"
              height="88"
            ></v-skeleton-loader>
            <div v-else class="d-flex align-center">
              <v-icon color="info" size="48" class="mr-4">mdi-tools</v-icon>
              <div class="flex-grow-1">
                <div class="text-h4">{{ toolsStore.toolsCount }}</div>
                <div class="text-subtitle-1 text-grey">Tools</div>
                <div class="text-caption text-grey">
                  <v-icon size="12" color="success">mdi-circle</v-icon>
                  {{ toolsStore.runningTools.length }} Active
                  <v-icon size="12" color="grey" class="ml-2"
                    >mdi-circle</v-icon
                  >
                  {{ toolsStore.stoppedTools.length }} Inactive
                </div>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Docker Monitoring Section -->
    <v-row class="mt-4">
      <!-- 1. CPU Load -->
      <v-col cols="12" md="6" lg="3">
        <v-card rounded="0" elevation="1" class="pa-4 d-flex flex-column" style="min-height: 350px;">
          <div class="text-subtitle-2 text-grey mb-2">CPU Load</div>
          
          <v-skeleton-loader v-if="dockerStore.loading" type="card" height="260"></v-skeleton-loader>
          
          <template v-else>
            <!-- Circular Progress - Centered with flex-grow -->
            <div class="flex-grow-1 d-flex flex-column justify-center align-center">
              <v-progress-circular
                :model-value="parseFloat(dockerStore.stats.cpu.percent)"
                :size="230"
                :width="20"
                :color="dockerStore.cpuColor"
              >
                <div class="text-center">
                  <div class="text-h4 font-weight-bold">{{ dockerStore.stats.cpu.percent }}%</div>
                  <div class="text-caption text-grey">CPU Load</div>
                </div>
              </v-progress-circular>
            </div>
            
            <!-- Breakdown - Stuck to bottom -->
            <v-row dense class="text-caption justify-center mt-4">
              <v-col cols="3" class="text-center">
                <div class="text-grey">System</div>
                <div class="font-weight-bold">{{ dockerStore.stats.cpu.breakdown.system }}%</div>
              </v-col>
              <v-col cols="3" class="text-center">
                <div class="text-grey">User</div>
                <div class="font-weight-bold">{{ dockerStore.stats.cpu.breakdown.user }}%</div>
              </v-col>
              <v-col cols="3" class="text-center">
                <div class="text-grey">Nice</div>
                <div class="font-weight-bold">{{ dockerStore.stats.cpu.breakdown.nice }}%</div>
              </v-col>
              <v-col cols="3" class="text-center">
                <div class="text-grey">Idle</div>
                <div class="font-weight-bold">{{ dockerStore.stats.cpu.breakdown.idle }}%</div>
              </v-col>
            </v-row>
          </template>
        </v-card>
      </v-col>

      <!-- 2. CPU Usage History -->
      <v-col cols="12" md="6" lg="3">
        <v-card
          rounded="0"
          elevation="1"
          class="pa-4"
          style="min-height: 350px"
        >
          <div class="d-flex justify-space-between align-center mb-2">
            <div class="text-subtitle-2 text-grey">CPU Usage History</div>
            <div
              class="text-h6 font-weight-bold"
              :style="{ color: dockerStore.cpuColor }"
            >
              {{ dockerStore.stats.cpu.percent }}%
            </div>
          </div>

          <v-skeleton-loader
            v-if="dockerStore.loading" type="image" height="260"></v-skeleton-loader>

          <template v-else>
            <v-sparkline
              :model-value="dockerStore.cpuHistory.length > 0 ? dockerStore.cpuHistory : [0]"
              :gradient="['#4CAF50', '#81C784', '#4CAF50']"
              :line-width="3"
              :padding="8"
              :smooth="10"
              :fill="true"
              auto-draw
              height="200"
            ></v-sparkline>

            <v-row dense class="mt-2 text-caption">
              <v-col cols="4">
                <div class="text-center">
                  <div class="text-grey">Min</div>
                  <div class="font-weight-bold">
                    {{ dockerStore.cpuHistoryStats.min }}%
                  </div>
                </div>
              </v-col>
              <v-col cols="4">
                <div class="text-center">
                  <div class="text-grey">Avg</div>
                  <div class="font-weight-bold">
                    {{ dockerStore.cpuHistoryStats.avg }}%
                  </div>
                </div>
              </v-col>
              <v-col cols="4">
                <div class="text-center">
                  <div class="text-grey">Max</div>
                  <div class="font-weight-bold">
                    {{ dockerStore.cpuHistoryStats.max }}%
                  </div>
                </div>
              </v-col>
            </v-row>
          </template>
        </v-card>
      </v-col>

      <!-- 3. Memory -->
      <v-col cols="12" md="6" lg="3">
        <v-card rounded="0" elevation="1" class="pa-4 d-flex flex-column" style="min-height: 350px;">
          <div class="text-subtitle-2 text-grey mb-2">Memory</div>
          
          <v-skeleton-loader v-if="dockerStore.loading" type="card" height="260"></v-skeleton-loader>
          
          <template v-else>
            <!-- Circular Progress - Centered with flex-grow -->
            <div class="flex-grow-1 d-flex flex-column justify-center align-center">
              <v-progress-circular
                :model-value="parseFloat(dockerStore.stats.memory.percent)"
                :size="230"
                :width="20"
                :color="dockerStore.memoryColor"
              >
                <div class="text-center">
                  <div class="text-h4 font-weight-bold">{{ dockerStore.stats.memory.percent }}%</div>
                  <div class="text-caption text-grey">Memory</div>
                </div>
              </v-progress-circular>
            </div>
            
            <!-- Breakdown - Stuck to bottom -->
            <v-row dense class="text-caption justify-center mt-4">
              <v-col cols="4" class="text-center">
                <div class="text-grey">Used</div>
                <div class="font-weight-bold">{{ dockerStore.stats.memory.used }} GB</div>
              </v-col>
              <v-col cols="4" class="text-center">
                <div class="text-grey">Available</div>
                <div class="font-weight-bold">{{ dockerStore.stats.memory.available }} GB</div>
              </v-col>
              <v-col cols="4" class="text-center">
                <div class="text-grey">Total</div>
                <div class="font-weight-bold">{{ dockerStore.stats.memory.total }} GB</div>
              </v-col>
            </v-row>
          </template>
        </v-card>
      </v-col>

      <!-- 4. Storage -->
      <v-col cols="12" md="6" lg="3">
        <v-card rounded="0" elevation="1" class="pa-4 d-flex flex-column" style="min-height: 350px;">
          <div class="text-subtitle-2 text-grey mb-2">Storage</div>
          
          <v-skeleton-loader v-if="dockerStore.loading" type="card" height="260"></v-skeleton-loader>
          
          <template v-else>
            <!-- Circular Progress - Centered with flex-grow -->
            <div class="flex-grow-1 d-flex flex-column justify-center align-center">
              <v-progress-circular
                :model-value="parseFloat(dockerStore.stats.storage.percent)"
                :size="230"
                :width="20"
                :color="dockerStore.storageColor"
              >
                <div class="text-center">
                  <div class="text-h4 font-weight-bold">{{ dockerStore.stats.storage.percent }}%</div>
                  <div class="text-caption text-grey">Used</div>
                </div>
              </v-progress-circular>
            </div>
            
            <!-- Breakdown - Stuck to bottom -->
            <v-row dense class="text-caption justify-center mt-4">
              <v-col cols="4" class="text-center">
                <div class="text-grey">Used</div>
                <div class="font-weight-bold">{{ dockerStore.stats.storage.used }} GB</div>
              </v-col>
              <v-col cols="4" class="text-center">
                <div class="text-grey">Available</div>
                <div class="font-weight-bold">{{ dockerStore.stats.storage.available }} GB</div>
              </v-col>
              <v-col cols="4" class="text-center">
                <div class="text-grey">Total</div>
                <div class="font-weight-bold">{{ dockerStore.stats.storage.total }} GB</div>
              </v-col>
            </v-row>
          </template>
        </v-card>
      </v-col>
    </v-row>

    <!-- Network Traffic -->
    <v-row class="mt-4">
      <v-col cols="12">
        <v-card rounded="0" elevation="1" class="pa-4">
          <!-- Header with Stats -->
          <div class="d-flex justify-space-between align-center mb-4">
            <div>
              <div class="text-subtitle-2 text-grey">Network Traffic</div>
              <div class="text-caption text-grey mt-1">Real-time network usage monitoring</div>
            </div>
            
            <!-- Stats Bar -->
            <template v-if="!dockerStore.loading">
              <div class="d-flex ga-4">
                <div class="text-center">
                  <v-icon color="success" size="small">mdi-arrow-down</v-icon>
                  <div class="text-caption text-grey">Download</div>
                  <div class="text-h6 font-weight-bold">{{ dockerStore.stats.network.rxSpeed || '0.00' }} MB/s</div>
                </div>
                <v-divider vertical></v-divider>
                <div class="text-center">
                  <v-icon color="warning" size="small">mdi-arrow-up</v-icon>
                  <div class="text-caption text-grey">Upload</div>
                  <div class="text-h6 font-weight-bold">{{ dockerStore.stats.network.txSpeed || '0.00' }} MB/s</div>
                </div>
                <v-divider vertical></v-divider>
                <div class="text-center">
                  <v-icon color="info" size="small">mdi-swap-vertical</v-icon>
                  <div class="text-caption text-grey">Total</div>
                  <div class="text-h6 font-weight-bold">{{ (parseFloat(dockerStore.stats.network.rxSpeed || 0) + parseFloat(dockerStore.stats.network.txSpeed || 0)).toFixed(2) }} MB/s</div>
                </div>
              </div>
            </template>
          </div>
          
          <v-skeleton-loader v-if="dockerStore.loading" type="paragraph" height="186"></v-skeleton-loader>
          
          <template v-else>
            <v-row>
              <v-col cols="12" md="6">
                <div class="text-caption text-grey mb-2 d-flex align-center">
                  <v-icon color="success" size="small" class="mr-1">mdi-arrow-down</v-icon>
                  Download History
                </div>
                <v-sparkline
                  :model-value="dockerStore.networkHistory.rx.length > 0 ? dockerStore.networkHistory.rx : [0]"
                  :gradient="['#4CAF50', '#81C784', '#4CAF50']"
                  :line-width="3"
                  :padding="8"
                  :smooth="10"
                  :fill="true"
                  auto-draw
                  height="35"
                ></v-sparkline>
              </v-col>
              <v-col cols="12" md="6">
                <div class="text-caption text-grey mb-2 d-flex align-center">
                  <v-icon color="warning" size="small" class="mr-1">mdi-arrow-up</v-icon>
                  Upload History
                </div>
                <v-sparkline
                  :model-value="dockerStore.networkHistory.tx.length > 0 ? dockerStore.networkHistory.tx : [0]"
                  :gradient="['#FF9800', '#FFB74D', '#FF9800']"
                  :line-width="3"
                  :padding="8"
                  :smooth="10"
                  :fill="true"
                  auto-draw
                  height="35"
                ></v-sparkline>
              </v-col>
            </v-row>
          </template>
        </v-card>
      </v-col>
    </v-row>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, onActivated, ref } from "vue";
import { useServicesStore } from "@/stores/services";
import { useProjectsStore } from "@/stores/projects";
import { useToolsStore } from "@/stores/tools";
import { useDockerStore } from "@/stores/docker";

const servicesStore = useServicesStore();
const projectsStore = useProjectsStore();
const toolsStore = useToolsStore();
const dockerStore = useDockerStore();

// Loading state
const loading = ref(true);

// Total containers (all services + projects + tools)
const totalContainers = computed(
  () =>
    servicesStore.servicesCount +
    projectsStore.projectsCount +
    toolsStore.toolsCount
);

const totalRunning = computed(
  () =>
    servicesStore.runningServices.length +
    projectsStore.runningProjects.length +
    toolsStore.runningTools.length
);

const totalStopped = computed(
  () =>
    servicesStore.stoppedServices.length +
    projectsStore.stoppedProjects.length +
    toolsStore.stoppedTools.length
);

// Auto-refresh interval
let refreshInterval = null;

onMounted(async () => {
  loading.value = true;
  try {
    await Promise.all([
      servicesStore.loadServices(),
      projectsStore.loadProjects(),
      toolsStore.loadTools(),
      dockerStore.loadStats(),
    ]);
  } finally {
    loading.value = false;
  }

  // Auto-refresh Docker stats every 2 seconds
  refreshInterval = setInterval(() => {
    dockerStore.loadStats();
  }, 2000);
});

// Reload data when returning to Dashboard
onActivated(async () => {
  loading.value = true;
  dockerStore.resetLoading(); // Reset to show skeleton on route change
  
  try {
    await Promise.all([
      servicesStore.loadServices(),
      projectsStore.loadProjects(),
      toolsStore.loadTools(),
      dockerStore.loadStats(),
    ]);
  } finally {
    loading.value = false;
  }
});

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval);
  }
});
</script>

<style scoped>
.cursor-pointer {
  cursor: pointer;
}
</style>
