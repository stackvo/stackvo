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
          <!-- Disabled tool - Enable button -->
          <v-btn 
            v-if="!item.configured" 
            size="small" 
            color="grey" 
            variant="tonal" 
            @click="enableTool(item.name)" 
            :loading="loadingTools[item.name] === 'enable'"
            :disabled="!!loadingTools[item.name]"
          >
            <v-icon start>mdi-power</v-icon>
            Disabled
          </v-btn>
          
          <!-- Enabled tool - Disable button -->
          <v-btn 
            v-else 
            size="small" 
            color="success" 
            variant="tonal" 
            @click="disableTool(item.name)" 
            :loading="loadingTools[item.name] === 'disable'"
            :disabled="!!loadingTools[item.name]"
          >
            <v-icon start>mdi-check-circle</v-icon>
            Enabled
          </v-btn>
        </template>

        <template v-slot:item.open="{ item }">
          <v-btn v-if="item.url && item.configured" block size="small" color="primary" variant="tonal" :href="item.url" target="_blank">
            <v-icon>mdi-open-in-new</v-icon>
          </v-btn>
        </template>

        <template v-slot:bottom></template>
      </v-data-table>
    </v-card>

    <!-- Progress Dialog -->
    <v-dialog v-model="showProgress" max-width="700" persistent>
      <v-card>
        <v-card-title class="d-flex align-center">
          <v-icon start>mdi-cog-sync</v-icon>
          {{ progressOperation }} {{ progressTool }}
          <v-spacer></v-spacer>
          <v-chip color="primary" variant="tonal">
            Processing...
          </v-chip>
        </v-card-title>

        <v-divider></v-divider>

        <v-card-text class="pa-4">
          <!-- Progress Steps -->
          <v-list density="compact">
            <v-list-item
              v-for="(step, index) in progressSteps"
              :key="index"
              :prepend-icon="
                step.status === 'done'
                  ? 'mdi-check-circle'
                  : step.status === 'running'
                  ? 'mdi-loading mdi-spin'
                  : 'mdi-circle-outline'
              "
              :class="{
                'text-success': step.status === 'done',
                'text-primary': step.status === 'running',
                'text-grey': step.status === 'pending',
              }"
            >
              <v-list-item-title>{{ step.message }}</v-list-item-title>
            </v-list-item>
          </v-list>
        </v-card-text>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, inject, nextTick } from 'vue';
import { useToolsStore } from '@/stores/tools';
import { io } from "socket.io-client";

const isNavigating = inject('isNavigating', ref(false));

const toolsStore = useToolsStore();
const toolSearch = ref('');
const loadingTools = ref({});

// WebSocket for realtime status updates
const socket = ref(null);

// Progress dialog for enable/disable operations
const showProgress = ref(false);
const progressOperation = ref('');
const progressTool = ref('');
const progressSteps = ref([]);

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
  loadingTools.value[containerName] = true;
  try {
    await toolsStore.startTool(containerName);
  } catch (error) {
    console.error('Failed to start tool:', error);
  } finally {
    loadingTools.value[containerName] = false;
  }
}

async function stopTool(containerName) {
  loadingTools.value[containerName] = true;
  try {
    await toolsStore.stopTool(containerName);
  } catch (error) {
    console.error('Failed to stop tool:', error);
  } finally {
    loadingTools.value[containerName] = false;
  }
}

async function restartTool(containerName) {
  loadingTools.value[containerName] = true;
  try {
    await toolsStore.restartTool(containerName);
  } catch (error) {
    console.error('Failed to restart tool:', error);
  } finally {
    loadingTools.value[containerName] = false;
  }
}

async function enableTool(toolName) {
  loadingTools.value[toolName] = 'enable';
  try {
    await toolsStore.enableTool(toolName);
  } catch (error) {
    console.error('Failed to enable tool:', error);
  } finally {
    delete loadingTools.value[toolName];
  }
}

async function disableTool(toolName) {
  loadingTools.value[toolName] = 'disable';
  try {
    await toolsStore.disableTool(toolName);
  } catch (error) {
    console.error('Failed to disable tool:', error);
  } finally {
    delete loadingTools.value[toolName];
  }
}

onMounted(async () => {
  await toolsStore.loadTools();
  
  // Connect to WebSocket
  socket.value = io();
  
  // Listen to tool events
  socket.value.on("tool:enabling", (data) => {
    console.log("Tool enabling:", data.tool);
    showProgress.value = true;
    progressOperation.value = "Enabling";
    progressTool.value = data.tool;
    progressSteps.value = [
      { message: "Updating .env configuration...", status: "running" },
      { message: "Regenerating templates...", status: "pending" },
      { message: "Stopping tools container...", status: "pending" },
      { message: "Removing old image...", status: "pending" },
      { message: "Rebuilding tools container...", status: "pending" },
      { message: "Starting tools container...", status: "pending" },
    ];
  });
  
  socket.value.on("tool:enabled", (data) => {
    console.log("Tool enabled:", data.tool);
    
    // Update progress steps
    progressSteps.value = [
      { message: "Updating .env configuration...", status: "done" },
      { message: "Regenerating templates...", status: "done" },
      { message: "Stopping tools container...", status: "done" },
      { message: "Removing old image...", status: "done" },
      { message: "Rebuilding tools container...", status: "done" },
      { message: "Starting tools container...", status: "done" },
    ];
    
    // Close dialog immediately
    showProgress.value = false;
    progressSteps.value = [];
    
    // Wait for dialog to disappear, then reload tools
    nextTick(async () => {
      await toolsStore.loadTools();
    });
  });
  
  socket.value.on("tool:disabling", (data) => {
    console.log("Tool disabling:", data.tool);
    showProgress.value = true;
    progressOperation.value = "Disabling";
    progressTool.value = data.tool;
    progressSteps.value = [
      { message: "Stopping tools container...", status: "running" },
      { message: "Removing old image...", status: "pending" },
      { message: "Updating .env configuration...", status: "pending" },
      { message: "Regenerating templates...", status: "pending" },
      { message: "Rebuilding tools container...", status: "pending" },
      { message: "Starting tools container...", status: "pending" },
    ];
  });
  
  socket.value.on("tool:disabled", (data) => {
    console.log("Tool disabled:", data.tool);
    
    // Update progress steps
    progressSteps.value = [
      { message: "Stopping tools container...", status: "done" },
      { message: "Removing old image...", status: "done" },
      { message: "Updating .env configuration...", status: "done" },
      { message: "Regenerating templates...", status: "done" },
      { message: "Rebuilding tools container...", status: "done" },
      { message: "Starting tools container...", status: "done" },
    ];
    
    // Close dialog immediately
    showProgress.value = false;
    progressSteps.value = [];
    
    // Wait for dialog to disappear, then reload tools
    nextTick(async () => {
      await toolsStore.loadTools();
    });
  });
  
  socket.value.on("tool:error", (data) => {
    console.error("Tool error:", data.tool, data.error);
  });
});

onUnmounted(() => {
  if (socket.value) {
    socket.value.disconnect();
  }
});
</script>
