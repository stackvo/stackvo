import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import axios from 'axios';

export const useDockerStore = defineStore('docker', () => {
  // State
  const stats = ref({
    cpu: { 
      percent: '0.0', 
      breakdown: { user: '0.0', nice: '0.0', system: '0.0', idle: '100.0' }
    },
    memory: { total: '0.00', used: '0.00', available: '0.00', percent: '0.0' },
    storage: { total: '0', used: '0', available: '0', percent: '0' },
    network: { rx: '0.00', tx: '0.00' }
  });
  
  const cpuHistory = ref([]);
  const networkHistory = ref({ rx: [], tx: [] });
  const loading = ref(true);
  const initialLoading = ref(true);

  // Computed - Color helpers
  const cpuColor = computed(() => {
    const percent = parseFloat(stats.value.cpu.percent);
    if (percent < 50) return 'success';
    if (percent < 80) return 'warning';
    return 'error';
  });

  const memoryColor = computed(() => {
    const percent = parseFloat(stats.value.memory.percent);
    if (percent < 50) return 'info';
    if (percent < 80) return 'warning';
    return 'error';
  });

  const storageColor = computed(() => {
    const percent = parseFloat(stats.value.storage.percent);
    if (percent < 70) return 'primary';
    if (percent < 90) return 'warning';
    return 'error';
  });

  // Computed - History stats
  const cpuHistoryStats = computed(() => {
    if (cpuHistory.value.length === 0) {
      return { min: '0.0', avg: '0.0', max: '0.0' };
    }
    
    const min = Math.min(...cpuHistory.value);
    const max = Math.max(...cpuHistory.value);
    const avg = cpuHistory.value.reduce((a, b) => a + b, 0) / cpuHistory.value.length;
    
    return {
      min: min.toFixed(1),
      avg: avg.toFixed(1),
      max: max.toFixed(1)
    };
  });

  // Actions
  async function loadStats() {
    // Only show loading on initial load
    if (initialLoading.value) {
      loading.value = true;
    }
    
    try {
      const response = await axios.get('/api/docker/stats');
      if (response.data.success) {
        stats.value = response.data.data;
        
        // Update CPU history (keep last 30 points = 1 minute at 2s interval)
        cpuHistory.value.push(parseFloat(stats.value.cpu.percent));
        if (cpuHistory.value.length > 30) cpuHistory.value.shift();
        
        // Update Network history - use speed (MB/s) not total (GB)
        networkHistory.value.rx.push(parseFloat(stats.value.network.rxSpeed || 0));
        networkHistory.value.tx.push(parseFloat(stats.value.network.txSpeed || 0));
        if (networkHistory.value.rx.length > 30) {
          networkHistory.value.rx.shift();
          networkHistory.value.tx.shift();
        }
      }
    } catch (error) {
      console.error('Error loading Docker stats:', error);
    } finally {
      loading.value = false;
      initialLoading.value = false;
    }
  }

  function resetLoading() {
    initialLoading.value = true;
  }

  return {
    // State
    stats,
    cpuHistory,
    networkHistory,
    loading,
    
    // Computed
    cpuColor,
    memoryColor,
    storageColor,
    cpuHistoryStats,
    
    // Actions
    loadStats,
    resetLoading
  };
});
