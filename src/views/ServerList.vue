<template>
  <div class="server-list-page">
    <!-- 添加服务器配置表单 -->
    <div class="config-section">
      <div class="section-header">
        <h3 class="section-title">添加 OneBot 服务器</h3>
      </div>
      
      <form @submit.prevent="addServer" class="config-form">
        <div class="form-row">
          <div class="form-group">
            <label for="serverName" class="form-label">服务器名称</label>
            <input
              id="serverName"
              v-model="newServer.name"
              type="text"
              class="form-input"
              placeholder="例如：QQ机器人1"
              required
            />
          </div>
          
          <div class="form-group">
            <label for="serverHost" class="form-label">监听地址</label>
            <input
              id="serverHost"
              v-model="newServer.host"
              type="text"
              class="form-input"
              placeholder="127.0.0.1"
              required
            />
          </div>
        </div>
        
        <div class="form-row">
          <div class="form-group">
            <label for="serverPort" class="form-label">端口</label>
            <input
              id="serverPort"
              v-model.number="newServer.port"
              type="number"
              class="form-input"
              placeholder="8080"
              min="1024"
              max="65535"
              required
            />
          </div>
          
          <div class="form-group">
            <label for="accessToken" class="form-label">访问令牌（可选）</label>
            <input
              id="accessToken"
              v-model="newServer.accessToken"
              type="text"
              class="form-input"
              placeholder="留空则不验证"
            />
          </div>
        </div>
        
        <div class="form-actions">
          <button type="submit" class="btn-primary" :disabled="isLoading">
            <span v-if="isLoading">添加中...</span>
            <span v-else>➕ 添加服务器</span>
          </button>
        </div>
      </form>
    </div>

    <!-- 服务器列表 -->
    <div class="servers-section">
      <div class="section-header">
        <h3 class="section-title">服务器列表</h3>
        <div class="header-info">
          <span class="server-count">{{ servers.length }} 个服务器</span>
          <span v-if="configPath" class="config-path" :title="configPath">
            📁 配置文件: {{ configPath.split('\\').pop() || configPath.split('/').pop() }}
          </span>
        </div>
      </div>
      
      <div v-if="servers.length === 0" class="empty-state">
        <div class="empty-icon">🖥️</div>
        <p class="empty-text">暂无 OneBot 服务器</p>
        <p class="empty-subtitle">请添加您的第一个服务器配置</p>
      </div>
      
      <div v-else class="server-grid">
        <div
          v-for="server in servers"
          :key="server.id"
          class="server-card"
        >
          <!-- 状态指示器 -->
          <div class="status-indicator">
            <span 
              class="status-dot"
              :class="{
                'status-connected': server.status === 'connected',
                'status-connecting': server.status === 'connecting',
                'status-disconnected': server.status === 'disconnected'
              }"
            ></span>
          </div>
          
          <!-- 服务器信息 -->
          <div class="server-info">
            <h4 class="server-name">{{ server.name }}</h4>
            <p class="server-address">{{ server.host }}:{{ server.port }}</p>
            <p class="server-status">
              状态: {{ getStatusText(server.status) }}
              <span v-if="server.connections > 0" class="connection-count">
                ({{ server.connections }} 个连接)
              </span>
            </p>
          </div>
          
          <!-- 控制按钮 -->
          <div class="server-controls">
            <button
              @click="toggleServer(server)"
              class="btn-toggle"
              :class="{
                'btn-stop': server.status === 'connected' || server.status === 'connecting',
                'btn-start': server.status === 'disconnected'
              }"
              :disabled="server.status === 'connecting'"
            >
              <span v-if="server.status === 'connected'">⏹️ 停止</span>
              <span v-else-if="server.status === 'connecting'">⏳ 启动中</span>
              <span v-else>▶️ 启动</span>
            </button>
            
            <button @click="removeServer(server.id)" class="btn-remove" title="删除服务器">
              🗑️
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// 响应式数据
const servers = ref([]);
const isLoading = ref(false);
const configPath = ref('');

const newServer = reactive({
  name: '',
  host: '127.0.0.1',
  port: 8080,
  accessToken: ''
});

// 状态文本映射
const getStatusText = (status) => {
  const statusMap = {
    connected: '已连接',
    connecting: '连接中',
    disconnected: '未连接'
  };
  return statusMap[status] || '未知';
};

// 添加服务器
const addServer = async () => {
  if (isLoading.value) return;
  
  isLoading.value = true;
  try {
    // 调用后端添加服务器配置
    const result = await invoke('add_server_config', {
      name: newServer.name,
      host: newServer.host,
      port: newServer.port,
      accessToken: newServer.accessToken || null
    });
    
    // 转换为前端格式
    const serverConfig = {
      id: result.id,
      name: result.name,
      host: result.host,
      port: result.port,
      accessToken: result.access_token,
      status: 'disconnected',
      connections: 0,
      enabled: result.enabled,
      autoStart: result.auto_start,
      createdAt: result.created_at,
      updatedAt: result.updated_at
    };
    
    servers.value.push(serverConfig);
    
    // 重置表单
    newServer.name = '';
    newServer.host = '127.0.0.1';
    newServer.port = 8080;
    newServer.accessToken = '';
    
  } catch (error) {
    console.error('添加服务器失败:', error);
    alert('添加服务器失败: ' + error);
  } finally {
    isLoading.value = false;
  }
};

// 切换服务器状态
const toggleServer = async (server) => {
  if (server.status === 'connecting') return;
  
  if (server.status === 'connected') {
    // 停止服务器
    server.status = 'disconnected';
    server.connections = 0;
    
    try {
      // 调用后端停止服务器
      const result = await invoke('stop_onebot_server');
      console.log('服务器停止结果:', result);
      
      // 更新配置文件中的状态
      await invoke('set_server_enabled', {
        serverId: server.id,
        enabled: false
      });
      server.enabled = false;
      
    } catch (error) {
      console.error('停止服务器失败:', error);
      alert('停止服务器失败: ' + error);
    }
  } else {
    // 启动服务器
    server.status = 'connecting';
    
    try {
      const result = await invoke('start_onebot_server', {
        host: server.host,
        port: server.port,
        accessToken: server.accessToken
      });
      
      console.log('服务器启动结果:', result);
      
      // 更新配置文件中的状态
      await invoke('set_server_enabled', {
        serverId: server.id,
        enabled: true
      });
      server.enabled = true;
      
      // 状态会通过定时刷新自动更新
      
    } catch (error) {
      console.error('启动服务器失败:', error);
      server.status = 'disconnected';
      alert('启动服务器失败: ' + error);
    }
  }
};

// 删除服务器
const removeServer = async (serverId) => {
  const index = servers.value.findIndex(s => s.id === serverId);
  if (index > -1) {
    const server = servers.value[index];
    if (server.status === 'connected') {
      alert('请先停止服务器再删除');
      return;
    }
    
    try {
      // 从配置文件中删除
      await invoke('remove_server_config', {
        serverId: serverId
      });
      
      // 从前端列表中删除
      servers.value.splice(index, 1);
    } catch (error) {
      console.error('删除服务器失败:', error);
      alert('删除服务器失败: ' + error);
    }
  }
};

// 获取配置文件路径
const loadConfigPath = async () => {
  try {
    const result = await invoke('get_config_path');
    configPath.value = result;
    console.log('配置文件路径:', result);
  } catch (error) {
    console.error('获取配置文件路径失败:', error);
  }
};

// 从后端加载服务器配置
const loadServers = async () => {
  try {
    const result = await invoke('get_all_servers');
    
    // 检查服务器运行时状态
    let runtimeStatus = { isRunning: false, status: 'disconnected', connections: 0 };
    try {
      const [isRunning, statusStr, connectionCount] = await invoke('get_server_runtime_status');
      runtimeStatus = {
        isRunning,
        status: statusStr,
        connections: connectionCount
      };
    } catch (error) {
      console.error('获取运行时状态失败:', error);
    }
    
    // 转换为前端格式
    servers.value = result.map(server => {
      // 根据配置文件的enabled状态和实际运行状态来确定显示状态
      let status = 'disconnected';
      let connections = 0;
      
      if (server.enabled && runtimeStatus.isRunning) {
        // 如果配置文件说是启用的，且有服务器在运行，则显示运行状态
        status = runtimeStatus.status;
        connections = runtimeStatus.connections;
      } else if (server.enabled && !runtimeStatus.isRunning) {
        // 配置说应该启用但实际没运行，可能是启动失败了
        status = 'disconnected';
        connections = 0;
      }
      
      return {
        id: server.id,
        name: server.name,
        host: server.host,
        port: server.port,
        accessToken: server.access_token,
        status: status,
        connections: connections,
        enabled: server.enabled,
        autoStart: server.auto_start,
        createdAt: server.created_at,
        updatedAt: server.updated_at
      };
    });
    
    console.log('已加载服务器配置:', servers.value.length, '个');
    console.log('运行时状态:', runtimeStatus);
  } catch (error) {
    console.error('加载服务器配置失败:', error);
    // 失败时使用空列表
    servers.value = [];
  }
};

// 定期刷新状态
const refreshTimer = ref(null);

// 刷新服务器状态（不重新加载配置，只刷新运行状态）
const refreshServerStatus = async () => {
  try {
    const [isRunning, statusStr, connectionCount] = await invoke('get_server_runtime_status');
    
    // 更新所有启用的服务器状态
    servers.value.forEach(server => {
      if (server.enabled && isRunning) {
        server.status = statusStr;
        server.connections = connectionCount;
      } else if (server.enabled && !isRunning) {
        server.status = 'disconnected';
        server.connections = 0;
      }
    });
  } catch (error) {
    console.error('刷新服务器状态失败:', error);
  }
};

// 页面加载时初始化
onMounted(async () => {
  console.log('服务器列表页面已加载');
  
  // 等待一小段时间确保配置管理器已初始化
  setTimeout(async () => {
    await loadConfigPath();
    await loadServers();
    
    // 启动定期刷新（每5秒刷新一次状态）
    refreshTimer.value = setInterval(refreshServerStatus, 5000);
  }, 100);
});

// 页面卸载时清理定时器
onUnmounted(() => {
  if (refreshTimer.value) {
    clearInterval(refreshTimer.value);
    refreshTimer.value = null;
  }
});
</script>

<style scoped>
.server-list-page {
  max-width: 1000px;
  margin: 0 auto;
  padding: 0;
}

/* 配置表单区域 */
.config-section {
  background-color: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 15px;
  padding: 24px;
  margin-bottom: 24px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.section-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-title);
  margin: 0;
}

.header-info {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
}

.server-count {
  font-size: 14px;
  color: var(--text-primary);
  opacity: 0.7;
}

.config-path {
  font-size: 12px;
  color: var(--text-primary);
  opacity: 0.6;
  cursor: help;
  max-width: 300px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: opacity 0.3s ease;
}

.config-path:hover {
  opacity: 0.8;
}

/* 表单样式 */
.config-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.form-input {
  padding: 12px 16px;
  border: 1px solid var(--border-color);
  border-radius: 15px;
  font-size: 14px;
  transition: all 0.3s ease;
  background-color: var(--card-bg);
  color: var(--text-primary);
}

.form-input:focus {
  outline: none;
  border-color: var(--button-bg);
  box-shadow: 0 0 0 3px rgba(169, 195, 166, 0.1);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 8px;
}

/* 按钮样式 */
.btn-primary {
  background-color: var(--button-bg);
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 30px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 4px 12px rgba(169, 195, 166, 0.3);
}

.btn-primary:hover:not(:disabled) {
  background-color: var(--button-hover);
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(169, 195, 166, 0.4);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

/* 服务器列表区域 */
.servers-section {
  background-color: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 15px;
  padding: 24px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}

.empty-state {
  text-align: center;
  padding: 60px 20px;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty-text {
  font-size: 18px;
  color: var(--text-title);
  margin-bottom: 8px;
  font-weight: 500;
}

.empty-subtitle {
  font-size: 14px;
  color: var(--text-primary);
  opacity: 0.7;
}

/* 服务器卡片网格 */
.server-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

.server-card {
  position: relative;
  background-color: var(--bg-color);
  border: 1px solid var(--border-color);
  border-radius: 15px;
  padding: 20px;
  transition: all 0.3s ease;
}

.server-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
}

/* 状态指示器 */
.status-indicator {
  position: absolute;
  top: 16px;
  right: 16px;
}

.status-dot {
  display: inline-block;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background-color: #ccc;
  transition: all 0.3s ease;
}

.status-dot.status-connected {
  background-color: #52c41a;
  box-shadow: 0 0 8px rgba(82, 196, 26, 0.4);
}

.status-dot.status-connecting {
  background-color: #faad14;
  animation: pulse 1.5s infinite;
}

.status-dot.status-disconnected {
  background-color: #d9d9d9;
}

@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.5; }
  100% { opacity: 1; }
}

/* 服务器信息 */
.server-info {
  margin-right: 60px;
  margin-bottom: 16px;
}

.server-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-title);
  margin: 0 0 4px 0;
}

.server-address {
  font-size: 14px;
  color: var(--text-primary);
  margin: 0 0 8px 0;
  font-family: 'Courier New', monospace;
}

.server-status {
  font-size: 13px;
  color: var(--text-primary);
  opacity: 0.8;
  margin: 0;
}

.connection-count {
  color: var(--button-bg);
  font-weight: 500;
}

/* 控制按钮 */
.server-controls {
  display: flex;
  gap: 8px;
}

.btn-toggle {
  flex: 1;
  padding: 8px 16px;
  border: none;
  border-radius: 15px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-start {
  background-color: var(--button-bg);
  color: white;
}

.btn-start:hover {
  background-color: var(--button-hover);
}

.btn-stop {
  background-color: #ff7875;
  color: white;
}

.btn-stop:hover {
  background-color: #ff4d4f;
}

.btn-toggle:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-remove {
  padding: 8px 12px;
  border: none;
  border-radius: 15px;
  background-color: transparent;
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.3s ease;
  opacity: 0.6;
}

.btn-remove:hover {
  background-color: #ff7875;
  color: white;
  opacity: 1;
}

@media (max-width: 768px) {
  .form-row {
    grid-template-columns: 1fr;
  }
  
  .server-grid {
    grid-template-columns: 1fr;
  }
}
</style> 