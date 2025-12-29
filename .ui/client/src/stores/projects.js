import { defineStore } from 'pinia';
import axios from 'axios';

export const useProjectsStore = defineStore('projects', {
  state: () => ({
    projects: [],
    loading: false,
    error: null
  }),

  getters: {
    runningProjects: (state) => state.projects.filter(p => p.running),
    stoppedProjects: (state) => state.projects.filter(p => !p.running),
    projectsCount: (state) => state.projects.length
  },

  actions: {
    async loadProjects() {
      this.loading = true;
      this.error = null;
      
      try {
        const response = await axios.get('/api/projects');
        if (response.data.success) {
          this.projects = response.data.data.projects;
        }
      } catch (error) {
        this.error = error.message;
        console.error('Error loading projects:', error);
      } finally {
        this.loading = false;
      }
    },

    async startProject(containerName) {
      try {
        const response = await axios.post(`/api/projects/${containerName}/start`);
        return response.data;
      } catch (error) {
        console.error('Error starting project:', error);
        throw error;
      }
    },

    async stopProject(containerName) {
      try {
        const response = await axios.post(`/api/projects/${containerName}/stop`);
        return response.data;
      } catch (error) {
        console.error('Error stopping project:', error);
        throw error;
      }
    },

    async restartProject(containerName) {
      try {
        const response = await axios.post(`/api/projects/${containerName}/restart`);
        return response.data;
      } catch (error) {
        console.error('Error restarting project:', error);
        throw error;
      }
    },

    async createProject(projectData) {
      this.loading = true;
      try {
        const response = await axios.post('/api/projects/create', projectData);
        
        if (response.data.success) {
          // Reload projects to show the new one
          await this.loadProjects();
          return response.data;
        } else {
          throw new Error(response.data.message || 'Failed to create project');
        }
      } catch (error) {
        console.error('Failed to create project:', error);
        throw error;
      } finally {
        this.loading = false;
      }
    },

    async deleteProject(projectName) {
      this.loading = true;
      try {
        const response = await axios.delete(`/api/projects/${projectName}`);
        
        if (response.data.success) {
          // Reload projects to update the list
          await this.loadProjects();
          return response.data;
        } else {
          throw new Error(response.data.message || 'Failed to delete project');
        }
      } catch (error) {
        console.error('Failed to delete project:', error);
        throw error;
      } finally {
        this.loading = false;
      }
    },

    async buildProject(projectName) {
      this.loading = true;
      try {
        const response = await axios.post(`/api/projects/${projectName}/build`);
        
        if (response.data.success) {
          await this.loadProjects();
          return response.data;
        } else {
          throw new Error(response.data.message || 'Failed to build project');
        }
      } catch (error) {
        console.error('Failed to build project:', error);
        throw error;
      } finally {
        this.loading = false;
      }
    }
  }
});
