<template>
  <v-navigation-drawer
    :model-value="modelValue"
    @update:model-value="(val) => emit('update:modelValue', val)"
    location="right"
    temporary
    absolute
    width="750"
    elevation="24"
    style="position: fixed !important; right: 0; top: 0; height: 100vh; z-index: 2001;"
  >
    <v-card class="h-100 d-flex flex-column" rounded="0">
      <v-card-title class="d-flex align-center bg-primary">
        <v-icon start>mdi-folder-plus</v-icon>
        New Project
        <v-spacer></v-spacer>
        <v-btn icon="mdi-close" variant="text" @click="close">
          <v-icon>mdi-close</v-icon>
        </v-btn>
      </v-card-title>

      <v-divider></v-divider>

      <v-card-text class="flex-grow-1 overflow-y-auto pa-5">
        <v-form ref="formRef" @submit.prevent="handleSubmit">
          <v-row>
            <v-col cols="12">
              <v-text-field
                v-model="formData.name"
                label="Project Name"
                prepend-inner-icon="mdi-folder"
                variant="outlined"
                hint="Only alphanumeric, dash, and underscore allowed"
                persistent-hint
                :rules="[
                  v => !!v || 'Project name is required',
                  v => /^[a-zA-Z0-9\-_]+$/.test(v) || 'Invalid characters'
                ]"
                required
                @input="onProjectNameChange"
              ></v-text-field>
            </v-col>

            <v-col cols="6">
              <v-text-field
                :model-value="computedDomain"
                label="Domain"
                prepend-inner-icon="mdi-web"
                variant="outlined"
                readonly
                hint="Auto-generated from project name"
                persistent-hint
              ></v-text-field>
            </v-col>

            <v-col cols="6">
              <v-text-field
                v-model="formData.document_root"
                label="Document Root"
                prepend-inner-icon="mdi-folder-open"
                variant="outlined"
                hint="Relative path from project root"
                persistent-hint
                :rules="[v => !!v || 'Document root is required']"
                required
              ></v-text-field>
            </v-col>

            <v-col cols="6">
              <v-select
                v-model="formData.runtime"
                :items="['php', 'nodejs', 'python', 'ruby', 'golang']"
                label="Runtime"
                prepend-inner-icon="mdi-code-braces"
                variant="outlined"
                required
                hide-details
              ></v-select>
            </v-col>

            <v-col cols="6">
              <v-select
                v-if="formData.runtime"
                v-model="formData.version"
                :items="runtimeVersions"
                :label="formData.runtime.toUpperCase() + ' Version'"
                prepend-inner-icon="mdi-tag"
                variant="outlined"
                required
                hide-details
              ></v-select>
            </v-col>

            <v-col cols="12">
              <v-select
                v-model="formData.webserver"
                :items="['nginx', 'apache', 'caddy']"
                label="Web Server"
                prepend-inner-icon="mdi-server"
                variant="outlined"
                hide-details
                required
              ></v-select>
            </v-col>

            <v-col cols="12" v-if="formData.runtime === 'php'">
              <v-card variant="tonal">
                <v-card-title class="text-subtitle-2">
                  <v-icon start size="small">mdi-puzzle</v-icon>
                  PHP Extensions
                </v-card-title>
                <v-card-text style="max-height: 290px; overflow-y: auto;">
                  <v-row dense>
                    <v-col cols="6" v-for="ext in phpExtensions" :key="ext">
                      <v-checkbox
                        v-model="formData.extensions"
                        :label="ext"
                        :value="ext"
                        density="compact"
                        hide-details
                      ></v-checkbox>
                    </v-col>
                  </v-row>
                </v-card-text>
              </v-card>
            </v-col>
          </v-row>
        </v-form>
      </v-card-text>

      <v-divider></v-divider>

      <v-card-actions class="pa-4">
        <v-spacer></v-spacer>
        <v-btn variant="text" @click="close">Cancel</v-btn>
        <v-btn color="primary" variant="flat" @click="handleSubmit" :loading="loading">
          Create Project
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-navigation-drawer>
</template>

<script setup>
import { ref, computed, watch } from 'vue';

const props = defineProps({
  modelValue: Boolean
});

const emit = defineEmits(['update:modelValue', 'created']);

const formRef = ref(null);
const loading = ref(false);

const formData = ref({
  name: '',
  domain: '',
  document_root: 'public',
  runtime: 'php',
  version: '8.2',
  webserver: 'nginx',
  extensions: ['pdo', 'pdo_mysql', 'mysqli', 'gd', 'curl', 'zip', 'mbstring']
});

const phpExtensions = [
  'pdo', 'pdo_mysql', 'pdo_pgsql', 'mysqli', 'pgsql',
  'gd', 'imagick', 'curl', 'zip', 'mbstring', 'intl',
  'soap', 'xml', 'dom', 'simplexml', 'xmlwriter',
  'bcmath', 'gmp', 'opcache', 'apcu', 'redis', 'memcached'
];

const runtimeVersions = computed(() => {
  const versions = {
    php: ['8.0', '8.1', '8.2', '8.3', '8.4'],
    nodejs: ['18', '20', '22'],
    python: ['3.9', '3.10', '3.11', '3.12', '3.13'],
    ruby: ['3.1', '3.2', '3.3'],
    golang: ['1.21', '1.22', '1.23']
  };
  return versions[formData.value.runtime] || [];
});

const computedDomain = computed(() => {
  return formData.value.name ? `${formData.value.name}.loc` : '';
});

function onProjectNameChange() {
  formData.value.domain = computedDomain.value;
}

function close() {
  emit('update:modelValue', false);
}

async function handleSubmit() {
  const { valid } = await formRef.value.validate();
  if (!valid) return;

  loading.value = true;
  try {
    const projectData = {
      ...formData.value,
      domain: computedDomain.value
    };
    
    emit('created', projectData);
  } catch (error) {
    console.error('Form submission error:', error);
  } finally {
    loading.value = false;
  }
}

// Reset form when drawer closes
watch(() => props.modelValue, (newVal) => {
  if (!newVal) {
    formData.value = {
      name: '',
      domain: '',
      document_root: 'public',
      runtime: 'php',
      version: '8.2',
      webserver: 'nginx',
      extensions: ['pdo', 'pdo_mysql', 'mysqli', 'gd', 'curl', 'zip', 'mbstring']
    };
    formRef.value?.resetValidation();
  }
});
</script>
