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
        <v-icon start>mdi-folder-multiple</v-icon>
        Projects
        <v-spacer></v-spacer>
        <p>
          {{ runningProjectsCount }} /
          {{ projectsStore.projects.length }} Running
        </p>
        <v-divider vertical class="mx-2"></v-divider>
        <v-btn icon="mdi-plus" variant="flat" @click="newProjectDrawer = true">
          <v-icon>mdi-plus</v-icon>
          <v-tooltip activator="parent" location="bottom"
            >New Project</v-tooltip
          >
        </v-btn>
        <v-btn
          icon="mdi-refresh"
          variant="flat"
          :loading="projectsStore.loading"
          @click.prevent="loadProjects"
        >
          <v-icon>mdi-refresh</v-icon>
          <v-tooltip activator="parent" location="bottom"
            >Refresh Projects</v-tooltip
          >
        </v-btn>
      </v-card-title>
      <v-divider></v-divider>

      <v-card-text class="pa-0">
        <v-text-field
          v-model="projectSearch"
          prepend-inner-icon="mdi-magnify"
          label="Search projects..."
          class="rounded-0"
          single-line
          hide-details
          clearable
        ></v-text-field>
      </v-card-text>

      <v-data-table
        :headers="projectHeaders"
        :items="projectsStore.projects"
        :search="projectSearch"
        :loading="projectsStore.loading"
        items-per-page="-1"
        class="elevation-0"
        show-expand
        fixed-header
        hover
        v-model:expanded="expandedProjects"
        density="compact"
        item-value="name"
        striped="even"
        hide-default-footer
        style="height: calc(100vh - 235px)"
      >
        <template v-slot:loading>
          <v-skeleton-loader type="table-row@30"></v-skeleton-loader>
        </template>

        <template v-slot:item.name="{ item }">
          <div class="font-weight-bold">
            <v-icon size="small" class="mr-2">mdi-folder</v-icon>
            {{ item.name }}
          </div>
        </template>

        <template v-slot:item.domain="{ item }">
          <div v-if="item.domain" class="d-flex align-center">
            <a
              :href="'https://' + item.domain"
              target="_blank"
              class="text-decoration-none"
            >
              <v-icon size="small">mdi-web</v-icon> {{ item.domain }}
            </a>
            <v-tooltip v-if="!item.dns_configured" location="top">
              <template v-slot:activator="{ props }">
                <v-icon v-bind="props" color="warning" size="small" class="ml-2"
                  >mdi-alert-circle</v-icon
                >
              </template>
              <div class="text-caption">
                <strong>No Host DNS record found.</strong><br />
                Add the following to /etc/hosts:<br />
                <code class="bg-grey-darken-3 pa-1"
                  >127.0.0.1 {{ item.domain }}</code
                >
              </div>
            </v-tooltip>
          </div>
          <span v-else class="text-grey">-</span>
        </template>

        <template v-slot:item.runtime="{ item }">
          <v-icon start>mdi-language-php</v-icon>
          PHP {{ item.php?.version || "N/A" }}
        </template>

        <template v-slot:item.webserver="{ item }">
          {{ item.webserver }}
        </template>

        <template v-slot:item.configuration="{ item }">
          <div v-if="item.configuration">
            <v-tooltip v-if="item.configuration.has_custom" location="top">
              <template v-slot:activator="{ props }">
                <v-chip
                  v-bind="props"
                  size="small"
                  variant="tonal"
                  color="success"
                  class="w-100"
                >
                  <v-icon start size="small">mdi-cog</v-icon>
                  Custom
                </v-chip>
              </template>
              <div class="text-caption">
                <strong>Custom Configuration Files:</strong><br />
                <span
                  v-for="(file, index) in item.configuration.files"
                  :key="index"
                >
                  • {{ file }}<br />
                </span>
              </div>
            </v-tooltip>
            <v-chip
              v-else
              size="small"
              variant="tonal"
              color="grey"
              class="w-100"
            >
              <v-icon start size="small">mdi-cog-outline</v-icon>
              Default
            </v-chip>
          </div>
        </template>

        <template v-slot:item.control="{ item }">
          <!-- Build button - if container doesn't exist -->
          <v-btn
            v-if="!item.id"
            block
            size="small"
            color="info"
            variant="tonal"
            :loading="loadingProjects[item.name] === 'build'"
            :disabled="!!loadingProjects[item.name]"
            @click="buildProject(item.name)"
          >
            <v-icon>mdi-hammer-wrench</v-icon>
          </v-btn>

          <!-- Stop button - if running -->
          <v-btn
            v-else-if="item.running"
            block
            size="small"
            color="error"
            variant="tonal"
            :loading="loadingProjects[item.name] === 'stop'"
            :disabled="!!loadingProjects[item.name]"
            @click="stopProject(item.name)"
          >
            <v-icon>mdi-stop</v-icon>
          </v-btn>

          <!-- Start button - if stopped -->
          <v-btn
            v-else
            block
            size="small"
            color="success"
            variant="tonal"
            :loading="loadingProjects[item.name] === 'start'"
            :disabled="!!loadingProjects[item.name]"
            @click="startProject(item.name)"
          >
            <v-icon>mdi-play</v-icon>
          </v-btn>
        </template>

        <template v-slot:item.restart="{ item }">
          <v-btn
            v-if="item.running"
            block
            size="small"
            color="warning"
            variant="tonal"
            :loading="loadingProjects[item.name] === 'restart'"
            :disabled="!!loadingProjects[item.name]"
            @click="restartProject(item.name)"
          >
            <v-icon>mdi-restart</v-icon>
          </v-btn>
        </template>

        <template v-slot:item.terminal="{ item }">
          <v-btn
            v-if="item.running"
            block
            size="small"
            color="info"
            variant="tonal"
            :loading="loadingProjects[item.name] === 'terminal'"
            :disabled="!!loadingProjects[item.name]"
            @click="openTerminal(item.name)"
          >
            <v-icon>mdi-console</v-icon>
          </v-btn>
        </template>

        <template v-slot:item.open="{ item }">
          <v-btn
            v-if="item.domain && item.running"
            block
            size="small"
            color="primary"
            variant="tonal"
            :href="'https://' + item.domain"
            target="_blank"
          >
            <v-icon>mdi-open-in-new</v-icon>
          </v-btn>
        </template>

        <template v-slot:item.delete="{ item }">
          <v-btn
            block
            size="small"
            color="error"
            variant="tonal"
            @click="openDeleteDialog(item)"
          >
            <v-icon>mdi-delete</v-icon>
          </v-btn>
        </template>

        <!-- Expanded Row Content -->
        <template v-slot:expanded-row="{ columns, item }">
          <tr>
            <td :colspan="columns.length" class="pa-4">
              <v-card elevation="0" class="pa-4">
                <v-row no-gutters>
                  <!-- Container Information -->
                  <v-col cols="12" md="3">
                    <v-card elevation="0" class="pa-3">
                      <div class="text-subtitle-2 mb-2 d-flex align-center">
                        <v-icon size="small" color="info" class="mr-2"
                          >mdi-docker</v-icon
                        >
                        Container Information
                      </div>
                      <v-list density="compact" class="bg-transparent">
                        <v-list-item>
                          <v-list-item-title class="text-caption text-grey"
                            >Container Name</v-list-item-title
                          >
                          <v-list-item-subtitle
                            class="text-body-2 font-weight-bold"
                          >
                            <v-chip
                              size="small"
                              variant="tonal"
                              color="primary"
                            >
                              <v-icon start size="small">mdi-docker</v-icon>
                              {{ item.containerName }}
                            </v-chip>
                          </v-list-item-subtitle>
                        </v-list-item>
                        <v-list-item>
                          <v-list-item-title class="text-caption text-grey"
                            >Status</v-list-item-title
                          >
                          <v-list-item-subtitle
                            class="text-body-2 font-weight-bold"
                          >
                            <v-chip
                              size="small"
                              variant="tonal"
                              :color="item.running ? 'success' : 'error'"
                            >
                              <v-icon start size="small">{{
                                item.running
                                  ? "mdi-check-circle"
                                  : "mdi-close-circle"
                              }}</v-icon>
                              {{ item.running ? "Running" : "Stopped" }}
                            </v-chip>
                          </v-list-item-subtitle>
                        </v-list-item>
                        <v-list-item v-if="item.image">
                          <v-list-item-title class="text-caption text-grey"
                            >Image</v-list-item-title
                          >
                          <v-list-item-subtitle class="text-body-2">
                            <v-chip size="small" variant="tonal" color="info">
                              <v-icon start size="small"
                                >mdi-package-variant</v-icon
                              >
                              {{ item.image }}
                            </v-chip>
                          </v-list-item-subtitle>
                        </v-list-item>
                      </v-list>
                    </v-card>
                  </v-col>

                  <!-- Project Configuration -->
                  <v-col cols="12" md="3">
                    <v-card elevation="0" class="pa-3">
                      <div class="text-subtitle-2 mb-2 d-flex align-center">
                        <v-icon size="small" color="primary" class="mr-2"
                          >mdi-folder-cog</v-icon
                        >
                        Project Configuration
                      </div>
                      <v-row dense>
                        <v-col cols="6">
                          <v-list density="compact" class="bg-transparent">
                            <v-list-item v-if="item.domain">
                              <v-list-item-title class="text-caption text-grey"
                                >Domain</v-list-item-title
                              >
                              <v-list-item-subtitle class="text-body-2">
                                <a
                                  :href="'https://' + item.domain"
                                  target="_blank"
                                  class="text-decoration-none"
                                >
                                  <v-chip
                                    size="small"
                                    variant="tonal"
                                    color="primary"
                                  >
                                    <v-icon start size="small">mdi-web</v-icon>
                                    {{ item.domain }}
                                  </v-chip>
                                </a>
                              </v-list-item-subtitle>
                            </v-list-item>
                            <v-list-item v-if="item.ssl_enabled !== undefined">
                              <v-list-item-title class="text-caption text-grey"
                                >SSL Status</v-list-item-title
                              >
                              <v-list-item-subtitle class="text-body-2">
                                <v-chip
                                  size="small"
                                  variant="tonal"
                                  :color="
                                    item.ssl_enabled ? 'success' : 'warning'
                                  "
                                >
                                  <v-icon start size="small">{{
                                    item.ssl_enabled
                                      ? "mdi-lock"
                                      : "mdi-lock-open-variant"
                                  }}</v-icon>
                                  {{
                                    item.ssl_enabled
                                      ? "Enabled (HTTPS)"
                                      : "Disabled (HTTP)"
                                  }}
                                </v-chip>
                              </v-list-item-subtitle>
                            </v-list-item>
                            <v-list-item
                              v-if="
                                item.urls && (item.urls.https || item.urls.http)
                              "
                            >
                              <v-list-item-title class="text-caption text-grey"
                                >Access URLs</v-list-item-title
                              >
                              <v-list-item-subtitle class="text-body-2">
                                <div class="d-flex flex-column ga-1">
                                  <a
                                    v-if="item.urls.https"
                                    :href="item.urls.https"
                                    target="_blank"
                                    class="text-decoration-none"
                                  >
                                    <v-chip
                                      size="small"
                                      variant="tonal"
                                      color="success"
                                    >
                                      <v-icon start size="small"
                                        >mdi-lock</v-icon
                                      >
                                      <code class="text-caption">{{
                                        item.urls.https
                                      }}</code>
                                    </v-chip>
                                  </a>
                                  <a
                                    v-if="item.urls.http"
                                    :href="item.urls.http"
                                    target="_blank"
                                    class="text-decoration-none"
                                  >
                                    <v-chip
                                      size="small"
                                      variant="tonal"
                                      color="warning"
                                    >
                                      <v-icon start size="small"
                                        >mdi-lock-open-variant</v-icon
                                      >
                                      <code class="text-caption">{{
                                        item.urls.http
                                      }}</code>
                                    </v-chip>
                                  </a>
                                </div>
                              </v-list-item-subtitle>
                            </v-list-item>
                            <v-list-item v-if="item.php && item.php.version">
                              <v-list-item-title class="text-caption text-grey"
                                >PHP Version</v-list-item-title
                              >
                              <v-list-item-subtitle
                                class="text-body-2 font-weight-bold"
                              >
                                <v-chip
                                  size="small"
                                  variant="tonal"
                                  color="success"
                                >
                                  <v-icon start size="small"
                                    >mdi-language-php</v-icon
                                  >
                                  PHP {{ item.php.version }}
                                </v-chip>
                              </v-list-item-subtitle>
                            </v-list-item>
                          </v-list>
                        </v-col>
                        <v-col cols="6">
                          <v-list density="compact" class="bg-transparent">
                            <v-list-item v-if="item.webserver">
                              <v-list-item-title class="text-caption text-grey"
                                >Web Server</v-list-item-title
                              >
                              <v-list-item-subtitle
                                class="text-body-2 font-weight-bold"
                              >
                                <v-chip
                                  size="small"
                                  variant="tonal"
                                  color="info"
                                >
                                  <v-icon start size="small">mdi-server</v-icon>
                                  {{ item.webserver }}
                                </v-chip>
                              </v-list-item-subtitle>
                            </v-list-item>
                            <v-list-item v-if="item.document_root">
                              <v-list-item-title class="text-caption text-grey"
                                >Document Root</v-list-item-title
                              >
                              <v-list-item-subtitle class="text-body-2">
                                <v-chip
                                  size="small"
                                  variant="tonal"
                                  color="grey"
                                >
                                  <v-icon start size="small">mdi-folder</v-icon>
                                  <code class="text-caption">{{
                                    item.document_root
                                  }}</code>
                                </v-chip>
                              </v-list-item-subtitle>
                            </v-list-item>
                            <v-list-item
                              v-if="
                                item.project_path &&
                                item.project_path.container_path
                              "
                            >
                              <v-list-item-title class="text-caption text-grey"
                                >Container Path</v-list-item-title
                              >
                              <v-list-item-subtitle class="text-body-2">
                                <v-chip
                                  size="small"
                                  variant="tonal"
                                  color="blue"
                                >
                                  <v-icon start size="small">mdi-docker</v-icon>
                                  <code class="text-caption">{{
                                    item.project_path.container_path
                                  }}</code>
                                </v-chip>
                              </v-list-item-subtitle>
                            </v-list-item>
                            <v-list-item
                              v-if="
                                item.project_path && item.project_path.host_path
                              "
                            >
                              <v-list-item-title class="text-caption text-grey"
                                >Host Path</v-list-item-title
                              >
                              <v-list-item-subtitle class="text-body-2">
                                <v-chip
                                  size="small"
                                  variant="tonal"
                                  color="purple"
                                >
                                  <v-icon start size="small"
                                    >mdi-folder-open</v-icon
                                  >
                                  <code class="text-caption">{{
                                    item.project_path.host_path
                                  }}</code>
                                </v-chip>
                              </v-list-item-subtitle>
                            </v-list-item>
                          </v-list>
                        </v-col>
                      </v-row>
                    </v-card>
                  </v-col>

                  <!-- Log Information -->
                  <v-col cols="12" md="3">
                    <v-card elevation="0" class="pa-3">
                      <div class="text-subtitle-2 mb-2 d-flex align-center">
                        <v-icon size="small" color="orange" class="mr-2"
                          >mdi-text-box-outline</v-icon
                        >
                        Log Information
                      </div>
                      <v-list density="compact" class="bg-transparent">
                        <v-list-item v-if="item.logs && item.logs.web_access">
                          <v-list-item-title class="text-caption text-grey"
                            >Web Access Log</v-list-item-title
                          >
                          <v-list-item-subtitle class="text-body-2">
                            <div class="d-flex flex-column ga-1">
                              <v-chip size="small" variant="tonal" color="blue">
                                <v-icon start size="small">mdi-docker</v-icon>
                                <code class="text-caption">{{
                                  item.logs.web_access.container_path
                                }}</code>
                              </v-chip>
                              <v-chip
                                size="small"
                                variant="tonal"
                                color="orange"
                              >
                                <v-icon start size="small"
                                  >mdi-folder-open</v-icon
                                >
                                <code class="text-caption">{{
                                  item.logs.web_access.host_path
                                }}</code>
                              </v-chip>
                            </div>
                          </v-list-item-subtitle>
                        </v-list-item>
                        <v-list-item v-if="item.logs && item.logs.web_error">
                          <v-list-item-title class="text-caption text-grey"
                            >Web Error Log</v-list-item-title
                          >
                          <v-list-item-subtitle class="text-body-2">
                            <div class="d-flex flex-column ga-1">
                              <v-chip size="small" variant="tonal" color="blue">
                                <v-icon start size="small">mdi-docker</v-icon>
                                <code class="text-caption">{{
                                  item.logs.web_error.container_path
                                }}</code>
                              </v-chip>
                              <v-chip size="small" variant="tonal" color="red">
                                <v-icon start size="small"
                                  >mdi-alert-circle</v-icon
                                >
                                <code class="text-caption">{{
                                  item.logs.web_error.host_path
                                }}</code>
                              </v-chip>
                            </div>
                          </v-list-item-subtitle>
                        </v-list-item>
                        <v-list-item v-if="item.logs && item.logs.php_error">
                          <v-list-item-title class="text-caption text-grey"
                            >PHP Error Log</v-list-item-title
                          >
                          <v-list-item-subtitle class="text-body-2">
                            <div class="d-flex flex-column ga-1">
                              <v-chip size="small" variant="tonal" color="blue">
                                <v-icon start size="small">mdi-docker</v-icon>
                                <code class="text-caption">{{
                                  item.logs.php_error.container_path
                                }}</code>
                              </v-chip>
                              <v-chip
                                size="small"
                                variant="tonal"
                                color="error"
                              >
                                <v-icon start size="small">mdi-bug</v-icon>
                                <code class="text-caption">{{
                                  item.logs.php_error.host_path
                                }}</code>
                              </v-chip>
                            </div>
                          </v-list-item-subtitle>
                        </v-list-item>
                        <v-list-item
                          v-if="
                            !item.logs ||
                            (!item.logs.web_access &&
                              !item.logs.web_error &&
                              !item.logs.php_error)
                          "
                        >
                          <v-list-item-subtitle class="text-caption text-grey">
                            <v-icon size="small" class="mr-1"
                              >mdi-information-outline</v-icon
                            >
                            No log paths configured
                          </v-list-item-subtitle>
                        </v-list-item>
                      </v-list>
                    </v-card>
                  </v-col>

                  <!-- PHP Extensions -->
                  <v-col cols="12" md="3">
                    <v-card elevation="0" class="pa-3">
                      <div class="text-subtitle-2 mb-2 d-flex align-center">
                        <v-icon size="small" color="success" class="mr-2"
                          >mdi-puzzle</v-icon
                        >
                        PHP Extensions
                      </div>
                      <v-list density="compact" class="bg-transparent">
                        <v-list-item
                          v-if="
                            item.php &&
                            item.php.extensions &&
                            item.php.extensions.length > 0
                          "
                        >
                          <v-list-item-subtitle class="text-body-2">
                            <div class="d-flex flex-wrap ga-1">
                              <v-chip
                                v-for="ext in item.php.extensions"
                                :key="ext"
                                size="x-small"
                                variant="tonal"
                                color="success"
                              >
                                {{ ext }}
                              </v-chip>
                            </div>
                          </v-list-item-subtitle>
                        </v-list-item>
                        <v-list-item v-else>
                          <v-list-item-subtitle class="text-caption text-grey">
                            <v-icon size="small" class="mr-1"
                              >mdi-information-outline</v-icon
                            >
                            No extensions configured
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

    <!-- Delete Project Confirmation Dialog -->
    <v-dialog v-model="deleteDialog" max-width="500">
      <v-card>
        <v-card-title class="bg-error text-white">
          <v-icon start>mdi-alert</v-icon>
          Delete Project
        </v-card-title>

        <v-card-text class="pt-4">
          <v-alert type="warning" variant="tonal" class="mb-4">
            This action cannot be undone!
          </v-alert>

          <p class="text-body-1">
            Are you sure you want to delete the project
            <strong>{{ projectToDelete?.name }}</strong
            >?
          </p>

          <p class="text-body-2 text-grey mt-2">
            This will permanently delete:
          </p>
          <ul class="text-body-2 text-grey">
            <li>
              Project directory:
              <code>projects/{{ projectToDelete?.name }}</code>
            </li>
            <li>All project files and configurations</li>
          </ul>
        </v-card-text>

        <v-divider></v-divider>

        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn
            variant="text"
            @click="deleteDialog = false"
            :disabled="deleteLoading"
          >
            Cancel
          </v-btn>
          <v-btn
            color="error"
            variant="flat"
            @click="confirmDeleteProject"
            :loading="deleteLoading"
          >
            Delete Project
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Terminal Command Dialog -->
    <v-dialog v-model="terminalDialog" max-width="600">
      <v-card>
        <v-card-title class="bg-info text-white">
          <v-icon start>mdi-console</v-icon>
          Open Terminal
        </v-card-title>

        <v-card-text class="pt-4">
          <v-alert type="info" variant="tonal" class="mb-4">
            Copy and run this command in your terminal:
          </v-alert>

          <v-text-field
            :model-value="terminalCommand"
            readonly
            variant="outlined"
            density="comfortable"
            class="font-monospace"
          >
            <template v-slot:append-inner>
              <v-btn
                icon="mdi-content-copy"
                variant="text"
                size="small"
                @click="copyTerminalCommand"
              >
                <v-icon>mdi-content-copy</v-icon>
                <v-tooltip activator="parent" location="top"
                  >Copy Command</v-tooltip
                >
              </v-btn>
            </template>
          </v-text-field>
        </v-card-text>

        <v-divider></v-divider>

        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn variant="text" @click="terminalDialog = false"> Close </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Progress Dialog (Redesigned - Services Style) -->
    <v-dialog v-model="showBuildProgress" max-width="700" persistent>
      <v-card>
        <v-card-title class="d-flex align-center">
          <v-icon start>mdi-cog-sync</v-icon>
          Building {{ currentBuildProject }}
          <v-spacer></v-spacer>
          <v-chip
            :color="
              buildProgress[currentBuildProject]?.status === 'building'
                ? 'primary'
                : buildProgress[currentBuildProject]?.status === 'success'
                ? 'success'
                : 'error'
            "
            variant="tonal"
          >
            {{
              buildProgress[currentBuildProject]?.status === "building"
                ? "Processing..."
                : buildProgress[currentBuildProject]?.status === "success"
                ? "Success!"
                : "Failed"
            }}
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

          <!-- Build Logs (Show/Hide) - Only for Build operation -->
          <div v-if="buildLogs[currentBuildProject]?.length > 0" class="mt-4">
            <v-btn
              @click="showLogs = !showLogs"
              variant="outlined"
              size="small"
              block
            >
              <v-icon start>{{ showLogs ? "mdi-eye-off" : "mdi-eye" }}</v-icon>
              {{ showLogs ? "Hide Logs" : "Show Logs" }}
            </v-btn>

            <v-card
              v-if="showLogs"
              elevation="0"
              color="grey-darken-4"
              class="pa-2 mt-2"
            >
              <div
                style="
                  max-height: 300px;
                  overflow-y: auto;
                  font-family: monospace;
                  font-size: 12px;
                "
              >
                <div
                  v-for="(log, index) in buildLogs[currentBuildProject]"
                  :key="index"
                  :class="{
                    'text-red': log.type === 'stderr',
                    'text-blue': log.type === 'info',
                    'text-white': log.type === 'stdout',
                  }"
                  style="white-space: pre-wrap; word-break: break-all"
                >
                  {{ log.output }}
                </div>
              </div>
            </v-card>
          </div>
        </v-card-text>

        <v-divider></v-divider>

        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn
            v-if="buildProgress[currentBuildProject]?.status === 'error'"
            color="error"
            @click="showBuildProgress = false"
          >
            Close
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Snackbar -->
    <v-snackbar v-model="showSnackbar" :color="snackbarColor" :timeout="3000">
      {{ snackbarMessage }}
      <template v-slot:actions>
        <v-btn variant="text" @click="showSnackbar = false">Close</v-btn>
      </template>
    </v-snackbar>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, computed, inject, nextTick } from "vue";
import { useProjectsStore } from "@/stores/projects";
import { io } from "socket.io-client";

const projectsStore = useProjectsStore();
const newProjectDrawer = inject("newProjectDrawer");
const isNavigating = inject("isNavigating", ref(false));
const projectSearch = ref("");
const expandedProjects = ref([]);
const loadingProjects = ref({});
const showSnackbar = ref(false);
const snackbarMessage = ref("");
const snackbarColor = ref("success");
const deleteDialog = ref(false);
const projectToDelete = ref(null);
const deleteLoading = ref(false);
const terminalDialog = ref(false);
const terminalCommand = ref("");

// WebSocket for realtime build progress
const socket = ref(null);
const buildProgress = ref({});
const buildLogs = ref({});
const showBuildProgress = ref(false);
const currentBuildProject = ref(null);
const showLogs = ref(false); // Show/hide logs
const progressSteps = ref([]); // Progress steps for all operations

const projectHeaders = [
  { title: "Project", key: "name", sortable: true, align: "left" },
  { title: "Domain", key: "domain", sortable: true, align: "left" },
  { title: "Runtime", key: "runtime", sortable: true, align: "left" },
  { title: "Server", key: "webserver", sortable: true, align: "left" },
  {
    title: "Configuration",
    key: "configuration",
    sortable: true,
    align: "center",
    width: "100",
  },
  {
    title: "Stop/Start",
    key: "control",
    sortable: false,
    align: "center",
    width: "100",
  },
  {
    title: "Restart",
    key: "restart",
    sortable: false,
    align: "center",
    width: "100",
  },
  {
    title: "Terminal",
    key: "terminal",
    sortable: false,
    align: "center",
    width: "100",
  },
  {
    title: "Open",
    key: "open",
    sortable: false,
    align: "center",
    width: "100",
  },
  {
    title: "Delete",
    key: "delete",
    sortable: false,
    align: "center",
    width: "100",
  },
];

const runningProjectsCount = computed(
  () => projectsStore.runningProjects.length
);

async function loadProjects() {
  await projectsStore.loadProjects();
}

async function startProject(projectName) {
  loadingProjects.value[projectName] = "start";

  try {
    await projectsStore.startProject(projectName);
    // Success/error messages will be shown via WebSocket events
  } catch (error) {
    console.error("Failed to start project:", error);
    snackbarMessage.value = `Failed to start "${projectName}": ${error.message}`;
    snackbarColor.value = "error";
    showSnackbar.value = true;
  } finally {
    delete loadingProjects.value[projectName];
  }
}

async function stopProject(projectName) {
  loadingProjects.value[projectName] = "stop";

  try {
    await projectsStore.stopProject(projectName);
    // Success/error messages will be shown via WebSocket events
  } catch (error) {
    console.error("Failed to stop project:", error);
    snackbarMessage.value = `Failed to stop "${projectName}": ${error.message}`;
    snackbarColor.value = "error";
    showSnackbar.value = true;
  } finally {
    delete loadingProjects.value[projectName];
  }
}

async function buildProject(projectName) {
  try {
    // Start build (202 Accepted - immediate return)
    const response = await fetch(`/api/projects/${projectName}/build`, {
      method: "POST",
    });

    const data = await response.json();

    if (!data.success) {
      throw new Error(data.message);
    }

    // Progress will be shown via WebSocket events
    // No need to wait here, build runs in background
  } catch (error) {
    console.error("Failed to start build:", error);
    snackbarMessage.value = `Failed to start build for "${projectName}": ${error.message}`;
    snackbarColor.value = "error";
    showSnackbar.value = true;
  }
}

async function restartProject(projectName) {
  loadingProjects.value[projectName] = "restart";

  try {
    await projectsStore.restartProject(projectName);
    // Success/error messages will be shown via WebSocket events
  } catch (error) {
    console.error("Failed to restart project:", error);
    snackbarMessage.value = `Failed to restart "${projectName}": ${error.message}`;
    snackbarColor.value = "error";
    showSnackbar.value = true;
  } finally {
    delete loadingProjects.value[projectName];
  }
}

async function openTerminal(projectName) {
  const containerName = `stackvo-${projectName}`;
  const command = `docker exec -it ${containerName} bash`;

  // Show dialog with command
  terminalCommand.value = command;
  terminalDialog.value = true;
}

function copyTerminalCommand() {
  navigator.clipboard
    .writeText(terminalCommand.value)
    .then(() => {
      snackbarMessage.value = "Command copied to clipboard!";
      snackbarColor.value = "success";
      showSnackbar.value = true;
    })
    .catch(() => {
      snackbarMessage.value = "Failed to copy command";
      snackbarColor.value = "error";
      showSnackbar.value = true;
    });
}

function openDeleteDialog(project) {
  projectToDelete.value = project;
  deleteDialog.value = true;
}

async function confirmDeleteProject() {
  if (!projectToDelete.value) return;

  deleteLoading.value = true;
  deleteDialog.value = false; // Close confirmation dialog immediately

  try {
    // API call will trigger WebSocket events (project:deleting, project:deleted)
    // which will show the progress dialog automatically
    await projectsStore.deleteProject(projectToDelete.value.name);
    projectToDelete.value = null;
  } catch (error) {
    console.error("Failed to delete project:", error);
    // Error will be shown in progress dialog via project:error event
  } finally {
    deleteLoading.value = false;
  }
}

onMounted(async () => {
  await projectsStore.loadProjects();

  // Connect to WebSocket
  console.log('[SOCKET] Connecting to Socket.IO...');
  socket.value = io();
  
  // Connection event handlers
  socket.value.on('connect', () => {
    console.log('[SOCKET] Connected to Socket.IO server');
    console.log('[SOCKET] Socket ID:', socket.value.id);
  });
  
  socket.value.on('disconnect', (reason) => {
    console.log('[SOCKET] Disconnected from Socket.IO server. Reason:', reason);
  });
  
  socket.value.on('connect_error', (error) => {
    console.error('[SOCKET] Connection error:', error);
  });

  // Listen to build events
  socket.value.on("build:start", (data) => {
    console.log("[SOCKET EVENT] build:start received:", data);
    console.log("Build started:", data.project);
    currentBuildProject.value = data.project;
    buildProgress.value[data.project] = {
      status: "building",
      progress: 0,
    };
    buildLogs.value[data.project] = [];
    progressSteps.value = [
      { message: "Building Docker image...", status: "running" },
      { message: "Starting container...", status: "pending" },
    ];
    showLogs.value = false;
    showBuildProgress.value = true;
  });

  socket.value.on("build:progress", (data) => {
    if (!buildLogs.value[data.project]) {
      buildLogs.value[data.project] = [];
    }
    buildLogs.value[data.project].push({
      type: data.type,
      output: data.output,
      timestamp: new Date(),
    });

    // Update progress steps based on log count
    const logCount = buildLogs.value[data.project].length;
    if (logCount > 50 && progressSteps.value[0]?.status === "running") {
      progressSteps.value[0].status = "done";
      progressSteps.value[1].status = "running";
    }

    // Update progress (estimate based on log lines)
    const progress = Math.min(95, logCount * 2);
    buildProgress.value[data.project].progress = progress;
  });

  socket.value.on("build:success", (data) => {
    console.log("Build success:", data.project);
    progressSteps.value = [
      { message: "Building Docker image...", status: "done" },
      { message: "Starting container...", status: "done" },
    ];
    buildProgress.value[data.project] = {
      status: "success",
      progress: 100,
      message: data.message,
    };

    // Close dialog immediately on success
    showBuildProgress.value = false;
    currentBuildProject.value = null;
    progressSteps.value = [];

    // Wait for dialog to disappear from DOM, then reload table
    nextTick(async () => {
      await projectsStore.loadProjects();
    });
  });

  socket.value.on("build:error", (data) => {
    console.error("Build error:", data.project, data.error);
    buildProgress.value[data.project] = {
      status: "error",
      progress: 0,
      message: data.error,
    };
  });

  // Start/Stop/Restart/Delete events
  socket.value.on("project:starting", (data) => {
    console.log("Project starting:", data.project);
    currentBuildProject.value = data.project;
    buildProgress.value[data.project] = { status: "building" };
    progressSteps.value = [
      { message: "Starting containers...", status: "running" },
    ];
    showLogs.value = false;
    showBuildProgress.value = true;
  });

  socket.value.on("project:started", (data) => {
    console.log("Project started:", data.project);
    progressSteps.value = [
      { message: "Starting containers...", status: "done" },
    ];
    buildProgress.value[data.project] = { status: "success" };

    // Close dialog immediately on success
    showBuildProgress.value = false;
    currentBuildProject.value = null;
    progressSteps.value = [];

    // Wait for dialog to disappear from DOM, then reload table
    nextTick(async () => {
      await projectsStore.loadProjects();
    });
  });

  socket.value.on("project:stopping", (data) => {
    console.log("Project stopping:", data.project);
    currentBuildProject.value = data.project;
    buildProgress.value[data.project] = { status: "building" };
    progressSteps.value = [
      { message: "Stopping containers...", status: "running" },
    ];
    showLogs.value = false;
    showBuildProgress.value = true;
  });

  socket.value.on("project:stopped", (data) => {
    console.log("Project stopped:", data.project);
    progressSteps.value = [
      { message: "Stopping containers...", status: "done" },
    ];
    buildProgress.value[data.project] = { status: "success" };

    // Close dialog immediately on success
    showBuildProgress.value = false;
    currentBuildProject.value = null;
    progressSteps.value = [];

    // Wait for dialog to disappear from DOM, then reload table
    nextTick(async () => {
      await projectsStore.loadProjects();
    });
  });

  socket.value.on("project:restarting", (data) => {
    console.log("Project restarting:", data.project);
    currentBuildProject.value = data.project;
    buildProgress.value[data.project] = { status: "building" };
    progressSteps.value = [
      { message: "Stopping containers...", status: "running" },
      { message: "Starting containers...", status: "pending" },
    ];
    showLogs.value = false;
    showBuildProgress.value = true;
  });

  socket.value.on("project:restarted", (data) => {
    console.log("Project restarted:", data.project);
    progressSteps.value = [
      { message: "Stopping containers...", status: "done" },
      { message: "Starting containers...", status: "done" },
    ];
    buildProgress.value[data.project] = { status: "success" };

    // Close dialog immediately on success
    showBuildProgress.value = false;
    currentBuildProject.value = null;
    progressSteps.value = [];

    // Wait for dialog to disappear from DOM, then reload table
    nextTick(async () => {
      await projectsStore.loadProjects();
    });
  });

  socket.value.on("project:deleting", (data) => {
    console.log("Project deleting:", data.project);
    currentBuildProject.value = data.project;
    buildProgress.value[data.project] = { status: "building" };
    progressSteps.value = [
      { message: "Stopping containers...", status: "running" },
      { message: "Removing containers...", status: "pending" },
      { message: "Deleting project files...", status: "pending" },
    ];
    showLogs.value = false;
    showBuildProgress.value = true;
  });

  socket.value.on("project:deleted", (data) => {
    console.log("Project deleted:", data.project);
    progressSteps.value = [
      { message: "Stopping containers...", status: "done" },
      { message: "Removing containers...", status: "done" },
      { message: "Deleting project files...", status: "done" },
    ];
    buildProgress.value[data.project] = { status: "success" };

    // Close dialog immediately on success
    showBuildProgress.value = false;
    currentBuildProject.value = null;
    progressSteps.value = [];

    // Wait for dialog to disappear from DOM, then reload table
    nextTick(async () => {
      await projectsStore.loadProjects();
    });
  });

  socket.value.on("project:creating", (data) => {
    console.log("Project creating:", data.project);
    currentBuildProject.value = data.project;
    buildProgress.value[data.project] = { status: "building" };
    progressSteps.value = [
      { message: "Creating project directory...", status: "running" },
      { message: "Generating configuration files...", status: "pending" },
      { message: "Building Docker image...", status: "pending" },
    ];
    showLogs.value = false;
    showBuildProgress.value = true;
  });

  socket.value.on("project:created", (data) => {
    console.log("Project created:", data.project);
    progressSteps.value = [
      { message: "Creating project directory...", status: "done" },
      { message: "Generating configuration files...", status: "done" },
      { message: "Building Docker image...", status: "done" },
    ];
    buildProgress.value[data.project] = { status: "success" };

    // Close dialog immediately on success
    showBuildProgress.value = false;
    currentBuildProject.value = null;
    progressSteps.value = [];

    // Wait for dialog to disappear from DOM, then reload table
    nextTick(async () => {
      await projectsStore.loadProjects();
    });
  });

  socket.value.on("project:error", (data) => {
    console.error("Project error:", data.project, data.error);
    buildProgress.value[data.project] = { status: "error" };

    // Close dialog on error
    showBuildProgress.value = false;
    currentBuildProject.value = null;
    progressSteps.value = [];

    // Show error snackbar
    snackbarMessage.value = `Error: ${data.error}`;
    snackbarColor.value = "error";
    showSnackbar.value = true;
  });
});

onUnmounted(() => {
  if (socket.value) {
    socket.value.disconnect();
  }
});
</script>
