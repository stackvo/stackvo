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
        <v-icon start>mdi-tools</v-icon>
        Tools
        <v-spacer></v-spacer>
        <p>{{ runningTools }} / {{ totalTools }} Enabled</p>
        <v-divider vertical class="mx-2"></v-divider>
        <v-btn icon="mdi-refresh" variant="flat" @click="toolsStore.loadTools()">
          <v-icon>mdi-refresh</v-icon>
          <v-tooltip activator="parent" location="bottom">Refresh Tools</v-tooltip>
        </v-btn>
      </v-card-title>
      <v-divider></v-divider>

      <v-card-text class="pa-0">
        <v-text-field
          v-model="toolSearch"
          prepend-inner-icon="mdi-magnify"
          label="Search tools..."
          class="rounded-0"
          single-line
          hide-details
          clearable
        ></v-text-field>
      </v-card-text>

      <v-data-table
        :headers="toolHeaders"
        :items="tools"
        :search="toolSearch"
        :loading="toolsLoading"
        items-per-page="-1"
        class="elevation-0"
        fixed-header
        hover
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
          <div class="font-weight-bold">
            <v-icon size="small" class="mr-2">mdi-wrench</v-icon>
            {{ item.name }}
          </div>
        </template>

        <template v-slot:item.version="{ item }">
          <v-chip size="small" variant="tonal" color="info">
            {{ item.version }}
          </v-chip>
        </template>

        <template v-slot:item.url="{ item }">
          <div v-if="item.url">
            <a :href="item.url" target="_blank" class="text-decoration-none">
              <v-icon size="small">mdi-web</v-icon>
              {{ item.domain }}
            </a>
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

        <template v-slot:item.status="{ item }">
          <!-- Disabled tool - Enable button (icon only) -->
          <v-btn 
            v-if="!item.enabled" 
            icon
            size="small" 
            color="success" 
            variant="tonal" 
            @click="enableTool(item.name)" 
            :loading="loadingTools[item.name] === 'enable'"
            :disabled="!!loadingTools[item.name]"
          >
            <v-icon>mdi-power</v-icon>
          </v-btn>
          
          <!-- Enabled tool - Disable button (icon only) -->
          <v-btn 
            v-else 
            icon
            size="small" 
            color="orange-darken-2" 
            variant="tonal" 
            @click="disableTool(item.name)" 
            :loading="loadingTools[item.name] === 'disable'"
            :disabled="!!loadingTools[item.name]"
          >
            <v-icon>mdi-power-off</v-icon>
          </v-btn>
        </template>

        <template v-slot:item.open="{ item }">
          <v-btn v-if="item.url && item.enabled" block size="small" color="primary" variant="tonal" :href="item.url" target="_blank">
            <v-icon>mdi-open-in-new</v-icon>
          </v-btn>
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
import { useToolsStore } from '@/stores/tools';

const isNavigating = inject('isNavigating', ref(false));

const toolsStore = useToolsStore();
const toolSearch = ref('');
const loadingTools = ref({});
const showOverlay = ref(false);
const overlayMessage = ref('');

const toolHeaders = [
  { title: 'Tool', key: 'name', sortable: true, align: 'left' },
  { title: 'Version', key: 'version', sortable: true, align: 'center', width: '120' },
  { title: 'URL', key: 'url', sortable: true, align: 'left' },
  { title: 'Open', key: 'open', sortable: false, align: 'center', width: '100' },
  { title: 'Status', key: 'status', sortable: true, align: 'center', width: '150' },
];

const tools = computed(() => toolsStore.tools);
const toolsLoading = computed(() => toolsStore.loading);
const runningTools = computed(() => tools.value.filter(t => t.running).length);
const totalTools = computed(() => tools.value.length);

async function startTool(containerName) {
  showOverlay.value = true;
  overlayMessage.value = 'Starting tool...';
  loadingTools.value[containerName] = true;
  try {
    await toolsStore.startTool(containerName);
  } catch (error) {
    console.error('Failed to start tool:', error);
  } finally {
    loadingTools.value[containerName] = false;
    showOverlay.value = false;
  }
}

async function stopTool(containerName) {
  showOverlay.value = true;
  overlayMessage.value = 'Stopping tool...';
  loadingTools.value[containerName] = true;
  try {
    await toolsStore.stopTool(containerName);
  } catch (error) {
    console.error('Failed to stop tool:', error);
  } finally {
    loadingTools.value[containerName] = false;
    showOverlay.value = false;
  }
}

async function restartTool(containerName) {
  showOverlay.value = true;
  overlayMessage.value = 'Restarting tool...';
  loadingTools.value[containerName] = true;
  try {
    await toolsStore.restartTool(containerName);
  } catch (error) {
    console.error('Failed to restart tool:', error);
  } finally {
    loadingTools.value[containerName] = false;
    showOverlay.value = false;
  }
}

async function enableTool(toolName) {
  showOverlay.value = true;
  overlayMessage.value = `Enabling ${toolName}... (This may take 2-5 minutes - container rebuild required)`;
  loadingTools.value[toolName] = 'enable';
  try {
    await toolsStore.enableTool(toolName);
    await toolsStore.loadTools();
  } catch (error) {
    console.error('Failed to enable tool:', error);
  } finally {
    delete loadingTools.value[toolName];
    showOverlay.value = false;
  }
}

async function disableTool(toolName) {
  showOverlay.value = true;
  overlayMessage.value = `Disabling ${toolName}... (This may take 2-5 minutes - container rebuild required)`;
  loadingTools.value[toolName] = 'disable';
  try {
    await toolsStore.disableTool(toolName);
    await toolsStore.loadTools();
  } catch (error) {
    console.error('Failed to disable tool:', error);
  } finally {
    delete loadingTools.value[toolName];
    showOverlay.value = false;
  }
}

onMounted(async () => {
  await toolsStore.loadTools();
});
</script>
