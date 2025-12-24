<template>
  <div class="pa-2">
    <!-- Stats Cards -->
    <v-row>
      <v-col cols="12" md="3">
        <v-card rounded="0" elevation="1" hover class="cursor-pointer" @click="$emit('changeView', 'services')">
          <v-card-text style="min-height: 100px;">
            <div class="d-flex align-center">
              <v-icon color="primary" size="48" class="mr-4">mdi-server</v-icon>
              <div class="flex-grow-1">
                <div class="text-h4">{{ servicesStore.servicesCount }}</div>
                <div class="text-subtitle-1 text-grey">Services</div>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" md="3">
        <v-card rounded="0" elevation="1" hover class="cursor-pointer" @click="$emit('changeView', 'projects')">
          <v-card-text style="min-height: 100px;">
            <div class="d-flex align-center">
              <v-icon color="secondary" size="48" class="mr-4">mdi-folder-multiple</v-icon>
              <div class="flex-grow-1">
                <div class="text-h4">{{ projectsStore.projectsCount }}</div>
                <div class="text-subtitle-1 text-grey">Projects</div>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" md="3">
        <v-card rounded="0" elevation="1" hover>
          <v-card-text style="min-height: 100px;">
            <div class="d-flex align-center">
              <v-icon color="success" size="48" class="mr-4">mdi-check-circle</v-icon>
              <div class="flex-grow-1">
                <div class="text-h4">{{ totalRunning }}</div>
                <div class="text-subtitle-1 text-grey">Running</div>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" md="3">
        <v-card rounded="0" elevation="1" hover>
          <v-card-text style="min-height: 100px;">
            <div class="d-flex align-center">
              <v-icon color="error" size="48" class="mr-4">mdi-close-circle</v-icon>
              <div class="flex-grow-1">
                <div class="text-h4">{{ totalStopped }}</div>
                <div class="text-subtitle-1 text-grey">Stopped</div>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Docker Monitoring Section -->
    <v-row class="mt-4">
      <v-col cols="12">
        <v-card rounded="0" elevation="1">
          <v-card-title>Docker Monitoring</v-card-title>
          <v-card-text>
            <v-alert type="info" variant="tonal">
              Docker monitoring grafikleri yakında eklenecek (CPU, Memory, Storage, Network)
            </v-alert>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </div>
</template>

<script setup>
import { computed, onMounted } from 'vue';
import { useServicesStore } from '@/stores/services';
import { useProjectsStore } from '@/stores/projects';

const servicesStore = useServicesStore();
const projectsStore = useProjectsStore();

const totalRunning = computed(() => 
  servicesStore.runningServices.length + projectsStore.runningProjects.length
);

const totalStopped = computed(() => 
  servicesStore.stoppedServices.length + projectsStore.stoppedProjects.length
);

onMounted(async () => {
  await Promise.all([
    servicesStore.loadServices(),
    projectsStore.loadProjects()
  ]);
});
</script>

<style scoped>
.cursor-pointer {
  cursor: pointer;
}
</style>
