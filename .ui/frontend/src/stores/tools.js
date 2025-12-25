import { defineStore } from 'pinia';
import axios from 'axios';

export const useToolsStore = defineStore('tools', {
  state: () => ({
    tools: [],
    loading: false,
    error: null
  }),

  getters: {
    runningTools: (state) => state.tools.filter(t => t.running),
    stoppedTools: (state) => state.tools.filter(t => !t.running),
    toolsCount: (state) => state.tools.length
  },

  actions: {
    async loadTools() {
      this.loading = true;
      this.error = null;
      
      try {
        const response = await axios.get('/api/tools');
        if (response.data.success) {
          this.tools = response.data.data.tools;
        }
      } catch (error) {
        this.error = error.message;
        console.error('Error loading tools:', error);
      } finally {
        this.loading = false;
      }
    },

    async startTool(containerName) {
      try {
        const response = await axios.post(`/api/tools/${containerName}/start`);
        if (response.data.success) {
          await this.loadTools();
        }
        return response.data;
      } catch (error) {
        console.error('Error starting tool:', error);
        throw error;
      }
    },

    async stopTool(containerName) {
      try {
        const response = await axios.post(`/api/tools/${containerName}/stop`);
        if (response.data.success) {
          await this.loadTools();
        }
        return response.data;
      } catch (error) {
        console.error('Error stopping tool:', error);
        throw error;
      }
    },

    async restartTool(containerName) {
      try {
        const response = await axios.post(`/api/tools/${containerName}/restart`);
        if (response.data.success) {
          await this.loadTools();
        }
        return response.data;
      } catch (error) {
        console.error('Error restarting tool:', error);
        throw error;
      }
    },

    async enableTool(toolName) {
      try {
        // 5 minute timeout for container rebuild
        const response = await axios.post(`/api/tools/${toolName}/enable`, {}, {
          timeout: 300000 // 5 minutes
        });
        if (response.data.success) {
          await this.loadTools();
        }
        return response.data;
      } catch (error) {
        console.error('Error enabling tool:', error);
        throw error;
      }
    },

    async disableTool(toolName) {
      try {
        // 5 minute timeout for container rebuild
        const response = await axios.post(`/api/tools/${toolName}/disable`, {}, {
          timeout: 300000 // 5 minutes
        });
        if (response.data.success) {
          await this.loadTools();
        }
        return response.data;
      } catch (error) {
        console.error('Error disabling tool:', error);
        throw error;
      }
    }
  }
});
