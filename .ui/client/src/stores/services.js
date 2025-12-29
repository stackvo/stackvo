import { defineStore } from 'pinia';
import axios from 'axios';

export const useServicesStore = defineStore('services', {
  state: () => ({
    services: [],
    loading: false,
    error: null
  }),

  getters: {
    runningServices: (state) => state.services.filter(s => s.running),
    stoppedServices: (state) => state.services.filter(s => !s.running),
    servicesCount: (state) => state.services.length
  },

  actions: {
    async loadServices() {
      this.loading = true;
      this.error = null;
      
      try {
        const response = await axios.get('/api/services');
        if (response.data.success) {
          this.services = response.data.data.services;
        }
      } catch (error) {
        this.error = error.message;
        console.error('Error loading services:', error);
      } finally {
        this.loading = false;
      }
    },

    async startService(containerName) {
      try {
        const response = await axios.post(`/api/services/${containerName}/start`);
        if (response.data.success) {
          await this.loadServices();
        }
        return response.data;
      } catch (error) {
        console.error('Error starting service:', error);
        throw error;
      }
    },

    async stopService(containerName) {
      try {
        const response = await axios.post(`/api/services/${containerName}/stop`);
        if (response.data.success) {
          await this.loadServices();
        }
        return response.data;
      } catch (error) {
        console.error('Error stopping service:', error);
        throw error;
      }
    },

    async restartService(containerName) {
      try {
        const response = await axios.post(`/api/services/${containerName}/restart`);
        if (response.data.success) {
          await this.loadServices();
        }
        return response.data;
      } catch (error) {
        console.error('Error restarting service:', error);
        throw error;
      }
    },

    async enableService(serviceName) {
      try {
        const response = await axios.post(`/api/services/${serviceName}/enable`);
        return response.data;
      } catch (error) {
        console.error('Error enabling service:', error);
        throw error;
      }
    },

    async disableService(serviceName) {
      try {
        const response = await axios.post(`/api/services/${serviceName}/disable`);
        return response.data;
      } catch (error) {
        console.error('Error disabling service:', error);
        throw error;
      }
    }
  }
});
