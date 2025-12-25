<template>
  <div class="pa-2">
    <v-card rounded="0" elevation="0">
      <!-- Loading Progress -->
      <v-progress-linear
        v-if="isNavigating"
        indeterminate
        color="success"
        height="3"
        absolute
        top
      ></v-progress-linear>
      
      <v-card-title class="d-flex align-center">
        <v-icon start>mdi-server-network</v-icon>
        Services
        <v-spacer></v-spacer>
        <p>{{ runningServicesCount }} / {{ servicesStore.services.length }} Running</p>
        <v-divider vertical class="ms-4 me-3"></v-divider>
        <v-btn icon="mdi-refresh" variant="flat" :loading="servicesStore.loading" @click.prevent="loadServices">
          <v-icon>mdi-refresh</v-icon>
          <v-tooltip activator="parent" location="bottom">Refresh Services</v-tooltip>
        </v-btn>
      </v-card-title>
      <v-divider></v-divider>

      <v-card-text class="pa-0">
        <v-text-field
          v-model="serviceSearch"
          prepend-inner-icon="mdi-magnify"
          label="Search services..."
          class="rounded-0"
          single-line
          hide-details
          clearable
        ></v-text-field>
      </v-card-text>

      <v-data-table
        :headers="serviceHeaders"
        :items="servicesStore.services"
        :search="serviceSearch"
        :loading="servicesStore.loading"
        items-per-page="-1"
        class="elevation-0"
        show-expand
        fixed-header
        v-model:expanded="expandedServices"
        density="compact"
        item-value="name"
        striped="even"
        hide-default-footer
        style="height: calc(100vh - 235px)"
      >
        <template v-slot:loading>
          <v-skeleton-loader type="table-row@20"></v-skeleton-loader>
        </template>

        <template v-slot:item.name="{ item }">
          <div class="font-weight-bold">{{ item.name }}</div>
        </template>

        <template v-slot:item.container_name="{ item }">
          <v-chip size="small" variant="tonal" color="grey">
            <v-icon start size="small">mdi-docker</v-icon>
            {{ item.containerName }}
          </v-chip>
        </template>

        <template v-slot:item.domain="{ item }">
          <div v-if="item.domain" class="d-flex align-center">
            <span>{{ item.domain }}</span>
            <v-tooltip v-if="!item.dns_configured" location="top">
              <template v-slot:activator="{ props }">
                <v-icon v-bind="props" color="warning" size="small" class="ml-2">mdi-alert-circle</v-icon>
              </template>
              <div class="text-caption">
                <strong>No Host DNS record found.</strong><br>
                Add the following to /etc/hosts:<br>
                <code class="bg-grey-darken-3 pa-1">127.0.0.1 {{ item.domain }}</code>
              </div>
            </v-tooltip>
          </div>
          <span v-else class="text-grey">-</span>
        </template>

        <template v-slot:item.version="{ item }">
          {{ item.image ? item.image.split(':')[1] || 'latest' : '-' }}
        </template>

        <template v-slot:item.status="{ item }">
          <!-- Disabled service - Enable button (icon only) -->
          <v-btn 
            v-if="!item.enabled" 
            icon
            size="small" 
            color="success" 
            variant="tonal" 
            @click="enableService(item.name)" 
            :loading="loadingServices[item.name] === 'enable'"
            :disabled="!!loadingServices[item.name]"
          >
            <v-icon>mdi-power</v-icon>
          </v-btn>
          
          <!-- Enabled service - Disable button (icon only) -->
          <v-btn 
            v-else 
            icon
            size="small" 
            color="orange-darken-2" 
            variant="tonal" 
            @click="disableService(item.name)" 
            :loading="loadingServices[item.name] === 'disable'"
            :disabled="!!loadingServices[item.name]"
          >
            <v-icon>mdi-power-off</v-icon>
          </v-btn>
        </template>

        <template v-slot:item.control="{ item }">
          <!-- Only show Start/Stop for enabled services -->
          <template v-if="item.enabled">
            <!-- Running - Stop button -->
            <v-btn 
              v-if="item.running" 
              block 
              size="small" 
              color="error" 
              variant="tonal" 
              @click="stopService(item.containerName)" 
              :loading="loadingServices[item.containerName] === 'stop'"
              :disabled="!!loadingServices[item.containerName]"
            >
              <v-icon>mdi-stop</v-icon>
            </v-btn>
            
            <!-- Stopped - Start button -->
            <v-btn 
              v-else 
              block 
              size="small" 
              color="success" 
              variant="tonal" 
              @click="startService(item.containerName)" 
              :loading="loadingServices[item.containerName] === 'start'"
              :disabled="!!loadingServices[item.containerName]"
            >
              <v-icon>mdi-play</v-icon>
            </v-btn>
          </template>
        </template>

        <template v-slot:item.restart="{ item }">
          <!-- Only show Restart for enabled + running services -->
          <v-btn 
            v-if="item.enabled && item.running" 
            block 
            size="small" 
            color="warning" 
            variant="tonal" 
            @click="restartService(item.containerName)" 
            :loading="loadingServices[item.containerName] === 'restart'"
            :disabled="!!loadingServices[item.containerName]"
          >
            <v-icon>mdi-restart</v-icon>
          </v-btn>
        </template>

        <template v-slot:item.open="{ item }">
          <v-btn v-if="item.url && item.running" block size="small" color="primary" variant="tonal" :href="item.url" target="_blank">
            <v-icon>mdi-open-in-new</v-icon>
          </v-btn>
        </template>

        <!-- Expanded Row -->
        <template v-slot:expanded-row="{ columns, item }">
          <tr>
            <td :colspan="columns.length" class="pa-4">
              <v-card elevation="0" class="pa-4">
                <v-row>
                  <!-- Network Information -->
                  <v-col cols="12" md="4">
                    <v-card elevation="0" class="pa-3">
                      <div class="text-subtitle-2 mb-2 d-flex align-center">
                        <v-icon size="small" color="info" class="mr-2">mdi-network</v-icon>
                        Network Information
                      </div>
                      <v-list density="compact" class="bg-transparent">
                        <v-list-item>
                          <v-list-item-title class="text-caption text-grey">Container Name</v-list-item-title>
                          <v-list-item-subtitle class="text-body-2 font-weight-bold">
                            <v-chip size="small" variant="tonal" color="primary">
                              <v-icon start size="small">mdi-docker</v-icon>
                              {{ item.containerName }}
                            </v-chip>
                          </v-list-item-subtitle>
                        </v-list-item>
                        <v-list-item v-if="item.ports && item.ports.ip_address">
                          <v-list-item-title class="text-caption text-grey">Internal IP</v-list-item-title>
                          <v-list-item-subtitle class="text-body-2 font-weight-bold">
                            <v-chip size="small" variant="tonal" color="success">
                              <v-icon start size="small">mdi-ip</v-icon>
                              {{ item.ports.ip_address }}
                            </v-chip>
                          </v-list-item-subtitle>
                        </v-list-item>
                        <v-list-item v-if="item.ports && item.ports.network">
                          <v-list-item-title class="text-caption text-grey">Network</v-list-item-title>
                          <v-list-item-subtitle class="text-body-2 font-weight-bold">
                            <v-chip size="small" variant="tonal" color="info">
                              <v-icon start size="small">mdi-lan</v-icon>
                              {{ item.ports.network }}
                            </v-chip>
                          </v-list-item-subtitle>
                        </v-list-item>
                        <v-list-item v-if="item.ports && item.ports.gateway">
                          <v-list-item-title class="text-caption text-grey">Gateway</v-list-item-title>
                          <v-list-item-subtitle class="text-body-2 font-weight-bold">
                            <v-chip size="small" variant="tonal" color="warning">
                              <v-icon start size="small">mdi-router-network</v-icon>
                              {{ item.ports.gateway }}
                            </v-chip>
                          </v-list-item-subtitle>
                        </v-list-item>
                      </v-list>
                    </v-card>
                  </v-col>

                  <!-- Service Information -->
                  <v-col cols="12" md="4">
                    <v-card elevation="0" class="pa-3">
                      <div class="text-subtitle-2 mb-2 d-flex align-center">
                        <v-icon size="small" color="primary" class="mr-2">mdi-server</v-icon>
                        Service Information
                      </div>
                      <v-list density="compact" class="bg-transparent">
                        <v-list-item v-if="item.url">
                          <v-list-item-title class="text-caption text-grey">Service URL</v-list-item-title>
                          <v-list-item-subtitle class="text-body-2">
                            <a :href="item.url" target="_blank" class="text-decoration-none">
                              <v-chip size="small" variant="tonal" color="primary">
                                <v-icon start size="small">mdi-web</v-icon>
                                {{ item.url }}
                              </v-chip>
                            </a>
                          </v-list-item-subtitle>
                        </v-list-item>
                        <v-list-item v-if="item.ports && item.ports.ports && Object.keys(item.ports.ports).length > 0">
                          <v-list-item-title class="text-caption text-grey mb-2">Port Mappings</v-list-item-title>
                          <v-list-item-subtitle class="d-flex flex-wrap" style="white-space: normal; word-break: break-word;">
                            <v-chip 
                              v-for="(portInfo, dockerPort) in item.ports.ports" 
                              :key="dockerPort" 
                              size="small" 
                              variant="outlined" 
                              :color="portInfo.exposed ? 'success' : 'grey'" 
                              class="mr-2 mb-2"
                              style="white-space: nowrap;"
                            >
                              <v-icon start size="small">{{ portInfo.exposed ? 'mdi-check-network' : 'mdi-lan-disconnect' }}</v-icon>
                              <span style="white-space: nowrap;">{{ portInfo.docker_port }}<span v-if="portInfo.exposed"> → {{ portInfo.host_ip }}:{{ portInfo.host_port }}</span><span v-else class="text-grey ml-1">(internal)</span></span>
                            </v-chip>
                          </v-list-item-subtitle>
                        </v-list-item>
                        <v-list-item v-else-if="item.ports && item.ports.length > 0">
                          <v-list-item-title class="text-caption text-grey">Ports</v-list-item-title>
                          <v-list-item-subtitle class="text-body-2 font-weight-bold">
                            <v-chip 
                              v-for="port in item.ports" 
                              :key="port" 
                              size="small" 
                              variant="tonal" 
                              color="info" 
                              class="mr-2"
                            >
                              <v-icon start size="small">mdi-ethernet</v-icon>
                              {{ port }}
                            </v-chip>
                          </v-list-item-subtitle>
                        </v-list-item>

                        <!-- Credentials -->
                        <v-list-item v-if="item.credentials && Object.keys(item.credentials).length > 0">
                          <v-list-item-title class="text-caption text-grey mb-2">Credentials</v-list-item-title>
                          <v-list-item-subtitle>
                            <v-list density="compact" class="bg-transparent">
                              <v-list-item v-for="(value, key) in item.credentials" :key="key" class="px-0">
                                <template v-slot:prepend>
                                  <v-icon 
                                    size="small" 
                                    :color="key.includes('PASSWORD') || key.includes('PASS') ? 'error' : (key.includes('USER') || key.includes('USERNAME') ? 'success' : (key.includes('DATABASE') || key.includes('DB') ? 'info' : 'purple'))"
                                  >
                                    {{ key.includes('PASSWORD') || key.includes('PASS') ? 'mdi-lock' : 
                                       (key.includes('USER') || key.includes('USERNAME') ? 'mdi-account' : 
                                       (key.includes('DATABASE') || key.includes('DB') ? 'mdi-database' : 
                                       (key.includes('PORT') ? 'mdi-ethernet' : 'mdi-information'))) }}
                                  </v-icon>
                                </template>
                                <v-list-item-title class="text-caption">
                                  <span class="text-grey">{{ key }}:</span>
                                  <span class="font-weight-bold ml-1">{{ value }}</span>
                                </v-list-item-title>
                              </v-list-item>
                            </v-list>
                          </v-list-item-subtitle>
                        </v-list-item>
                      </v-list>
                    </v-card>
                  </v-col>

                  <!-- Log Information -->
                  <v-col cols="12" md="4">
                    <v-card elevation="0" class="pa-3">
                      <div class="text-subtitle-2 mb-2 d-flex align-center">
                        <v-icon size="small" color="orange" class="mr-2">mdi-text-box-outline</v-icon>
                        Log Information
                      </div>
                      <v-list density="compact" class="bg-transparent">
                        <v-list-item>
                          <v-list-item-title class="text-caption text-grey">Container Logs</v-list-item-title>
                          <v-list-item-subtitle class="text-body-2">
                            <v-chip size="small" variant="tonal" color="blue">
                              <v-icon start size="small">mdi-docker</v-icon>
                              <code class="text-caption">docker logs {{ item.containerName }}</code>
                            </v-chip>
                          </v-list-item-subtitle>
                        </v-list-item>
                        <v-list-item>
                          <v-list-item-title class="text-caption text-grey">Container Path</v-list-item-title>
                          <v-list-item-subtitle class="text-body-2">
                            <v-chip size="small" variant="tonal" color="blue">
                              <v-icon start size="small">mdi-docker</v-icon>
                              <code class="text-caption">/var/log/{{ item.name }}</code>
                            </v-chip>
                          </v-list-item-subtitle>
                        </v-list-item>
                        <v-list-item>
                          <v-list-item-title class="text-caption text-grey">Host Path</v-list-item-title>
                          <v-list-item-subtitle class="text-body-2">
                            <v-chip size="small" variant="tonal" color="orange">
                              <v-icon start size="small">mdi-folder</v-icon>
                              <code class="text-caption">logs/{{ item.name }}/</code>
                            </v-chip>
                          </v-list-item-subtitle>
                        </v-list-item>
                      </v-list>
                    </v-card>
                  </v-col>
                </v-row>
              </v-card>
            </td>
          </tr>
        </template>

        <template v-slot:bottom></template>
      </v-data-table>
    </v-card>

    <!-- Overlay -->
    <v-overlay v-model="showOverlay" class="align-center justify-center" :opacity="0.8">
      <v-card class="pa-8 text-center" min-width="300">
        <v-progress-circular indeterminate size="64" color="primary" class="mb-4"></v-progress-circular>
        <div class="text-h6">{{ overlayMessage }}</div>
      </v-card>
    </v-overlay>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, inject } from 'vue';
import { useServicesStore } from '@/stores/services';

const isNavigating = inject('isNavigating', ref(false));

const servicesStore = useServicesStore();
const serviceSearch = ref('');
const expandedServices = ref([]);
const loadingServices = ref({});
const showOverlay = ref(false);
const overlayMessage = ref('');

const serviceHeaders = [
  { title: 'Service', key: 'name', sortable: true },
  { title: 'Container Name', key: 'container_name', sortable: true, align: 'left' },
  { title: 'Domain', key: 'domain', sortable: true, align: 'left' },
  { title: 'Version', key: 'version', sortable: true, align: 'left' },
  { title: 'Stop/Start', key: 'control', sortable: false, align: 'center', width: '100' },
  { title: 'Restart', key: 'restart', sortable: false, align: 'center', width: '100' },
  { title: 'Open', key: 'open', sortable: false, align: 'center', width: '100' },
  { title: 'Status', key: 'status', sortable: true, align: 'center', width: '100' }
];

const runningServicesCount = computed(() => servicesStore.runningServices.length);

async function loadServices() {
  await servicesStore.loadServices();
}

async function startService(containerName) {
  showOverlay.value = true;
  overlayMessage.value = 'Starting service...';
  loadingServices.value[containerName] = 'start';
  try {
    await servicesStore.startService(containerName);
  } catch (error) {
    console.error('Failed to start service:', error);
  } finally {
    delete loadingServices.value[containerName];
    showOverlay.value = false;
  }
}

async function stopService(containerName) {
  showOverlay.value = true;
  overlayMessage.value = 'Stopping service...';
  loadingServices.value[containerName] = 'stop';
  try {
    await servicesStore.stopService(containerName);
  } catch (error) {
    console.error('Failed to stop service:', error);
  } finally {
    delete loadingServices.value[containerName];
    showOverlay.value = false;
  }
}

async function restartService(containerName) {
  showOverlay.value = true;
  overlayMessage.value = 'Restarting service...';
  loadingServices.value[containerName] = 'restart';
  try {
    await servicesStore.restartService(containerName);
  } catch (error) {
    console.error('Failed to restart service:', error);
  } finally {
    delete loadingServices.value[containerName];
    showOverlay.value = false;
  }
}

async function enableService(serviceName) {
  showOverlay.value = true;
  overlayMessage.value = `Enabling ${serviceName}...`;
  loadingServices.value[serviceName] = 'enable';
  try {
    await servicesStore.enableService(serviceName);
    // Reload services to update table
    await servicesStore.loadServices();
  } catch (error) {
    console.error('Failed to enable service:', error);
  } finally {
    delete loadingServices.value[serviceName];
    showOverlay.value = false;
  }
}

async function disableService(serviceName) {
  showOverlay.value = true;
  overlayMessage.value = `Disabling ${serviceName}...`;
  loadingServices.value[serviceName] = 'disable';
  try {
    await servicesStore.disableService(serviceName);
    // Reload services to update table
    await servicesStore.loadServices();
  } catch (error) {
    console.error('Failed to disable service:', error);
  } finally {
    delete loadingServices.value[serviceName];
    showOverlay.value = false;
  }
}

onMounted(async () => {
  await servicesStore.loadServices();
});
</script>
