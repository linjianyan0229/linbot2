<template>
  <div class="server-list-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <div class="title-section">
          <h1 class="page-title">服务器管理</h1>
          <p class="page-subtitle">管理您的 OneBot 服务器配置</p>
        </div>
        <div class="header-stats">
          <div class="stat-card">
            <div class="stat-icon">🖥️</div>
            <div class="stat-info">
              <div class="stat-number">{{ servers.length }}</div>
              <div class="stat-label">服务器总数</div>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-icon">🟢</div>
            <div class="stat-info">
              <div class="stat-number">{{ connectedServers }}</div>
              <div class="stat-label">在线服务器</div>
            </div>
          </div>
        </div>
      </div>

      <div class="header-actions">
        <div class="config-info" v-if="configPath">
          <span class="config-icon">📁</span>
          <span class="config-text" :title="configPath">
            {{ configPath.split('\\').pop() || configPath.split('/').pop() }}
          </span>
        </div>
        <button @click="showAddDialog = true" class="btn-primary">
          <span class="btn-icon">➕</span>
          <span class="btn-text">添加服务器</span>
        </button>
      </div>
    </div>

    <!-- 服务器内容区域 -->
    <div class="servers-content">
      <div v-if="servers.length === 0" class="empty-state">
        <div class="empty-illustration">
          <div class="empty-icon">🖥️</div>
          <div class="empty-decoration"></div>
        </div>
        <h3 class="empty-title">暂无服务器配置</h3>
        <p class="empty-description">开始添加您的第一个 OneBot 服务器，享受智能机器人管理体验</p>
        <button @click="showAddDialog = true" class="btn-primary btn-large">
          <span class="btn-icon">➕</span>
          <span class="btn-text">添加第一个服务器</span>
        </button>
      </div>

      <div v-else class="server-grid">
        <div
          v-for="server in servers"
          :key="server.id"
          class="server-card"
          :class="{ 'server-connected': server.status === 'connected' }"
        >
          <!-- 卡片头部 -->
          <div class="card-header">
            <div class="server-title">
              <h3 class="server-name">{{ server.name }}</h3>
              <div class="server-address">{{ server.host }}:{{ server.port }}</div>
            </div>
            <div class="status-badge" :class="`status-${server.status}`">
              <div class="status-dot"></div>
              <span class="status-text">{{ getStatusText(server.status) }}</span>
            </div>
          </div>

          <!-- 卡片内容 -->
          <div class="card-content">
            <div class="server-metrics">
              <div class="metric-item">
                <span class="metric-icon">🔗</span>
                <span class="metric-label">连接数</span>
                <span class="metric-value">{{ server.connections || 0 }}</span>
              </div>
              <div class="metric-item">
                <span class="metric-icon">⚙️</span>
                <span class="metric-label">状态</span>
                <span class="metric-value">{{ server.enabled ? '启用' : '禁用' }}</span>
              </div>
            </div>
          </div>

          <!-- 卡片操作 -->
          <div class="card-actions">
            <button
              @click="toggleServer(server)"
              class="btn-action"
              :class="{
                'btn-stop': server.status === 'connected' || server.status === 'connecting',
                'btn-start': server.status === 'disconnected'
              }"
              :disabled="server.status === 'connecting'"
            >
              <span v-if="server.status === 'connected'" class="btn-icon">⏹️</span>
              <span v-else-if="server.status === 'connecting'" class="btn-icon">⏳</span>
              <span v-else class="btn-icon">▶️</span>
              <span v-if="server.status === 'connected'" class="btn-text">停止</span>
              <span v-else-if="server.status === 'connecting'" class="btn-text">启动中</span>
              <span v-else class="btn-text">启动</span>
            </button>

            <button @click="removeServer(server.id)" class="btn-danger" title="删除服务器">
              <span class="btn-icon">🗑️</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 添加服务器弹窗 -->
    <div v-if="showAddDialog" class="modal-overlay" @click="closeDialog">
      <div class="modal-container" @click.stop>
        <div class="modal-header">
          <div class="modal-title-section">
            <h2 class="modal-title">添加 OneBot 服务器</h2>
            <p class="modal-subtitle">配置新的机器人服务器连接</p>
          </div>
          <button @click="closeDialog" class="btn-close">
            <span class="close-icon">✕</span>
          </button>
        </div>

        <!-- 错误信息提示 -->
        <div v-if="errorMessage" class="alert alert-error">
          <div class="alert-icon">⚠️</div>
          <div class="alert-content">
            <div class="alert-title">配置错误</div>
            <div class="alert-message">{{ errorMessage }}</div>
          </div>
        </div>
        
        <form @submit.prevent="addServer" class="modal-form">
          <div class="form-section">
            <h3 class="section-title">基本信息</h3>
            <div class="form-field">
              <label for="dialogServerName" class="field-label">服务器名称</label>
              <input
                id="dialogServerName"
                v-model="newServer.name"
                type="text"
                class="field-input"
                placeholder="例如：QQ机器人1"
                required
              />
              <div class="field-hint">为您的服务器起一个易识别的名称</div>
            </div>
          </div>

          <div class="form-section">
            <h3 class="section-title">连接配置</h3>
            <div class="form-grid">
              <div class="form-field">
                <label for="dialogServerHost" class="field-label">监听地址</label>
                <input
                  id="dialogServerHost"
                  v-model="newServer.host"
                  type="text"
                  class="field-input"
                  placeholder="127.0.0.1"
                  required
                />
                <div class="field-hint">服务器监听的IP地址</div>
              </div>

              <div class="form-field">
                <label for="dialogServerPort" class="field-label">端口号</label>
                <input
                  id="dialogServerPort"
                  v-model.number="newServer.port"
                  type="number"
                  class="field-input"
                  placeholder="8080"
                  min="1024"
                  max="65535"
                  required
                />
                <div class="field-hint">1024-65535 之间的端口号</div>
              </div>
            </div>
          </div>

          <div class="form-section">
            <h3 class="section-title">安全设置</h3>
            <div class="form-field">
              <label for="dialogAccessToken" class="field-label">访问令牌</label>
              <input
                id="dialogAccessToken"
                v-model="newServer.accessToken"
                type="text"
                class="field-input"
                placeholder="留空则不验证访问令牌"
              />
              <div class="field-hint">可选，用于验证客户端连接</div>
            </div>
          </div>

          <div class="modal-actions">
            <button type="button" @click="closeDialog" class="btn-secondary">
              <span class="btn-text">取消</span>
            </button>
            <button type="submit" class="btn-primary" :disabled="isLoading">
              <span v-if="isLoading" class="btn-icon">⏳</span>
              <span v-else class="btn-icon">➕</span>
              <span v-if="isLoading" class="btn-text">添加中...</span>
              <span v-else class="btn-text">添加服务器</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// 响应式数据
const servers = ref([]);
const isLoading = ref(false);
const configPath = ref('');
const showAddDialog = ref(false);
const errorMessage = ref('');

// 计算属性
const connectedServers = computed(() => {
  return servers.value.filter(server => server.status === 'connected').length;
});

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

// 关闭弹窗
const closeDialog = () => {
  showAddDialog.value = false;
  errorMessage.value = '';
  // 重置表单
  newServer.name = '';
  newServer.host = '127.0.0.1';
  newServer.port = 8080;
  newServer.accessToken = '';
};

// 添加服务器
const addServer = async () => {
  if (isLoading.value) return;
  
  // 清除之前的错误信息
  errorMessage.value = '';
  
  // 验证服务器名称是否重复
  const existingNameServer = servers.value.find(server => 
    server.name.toLowerCase() === newServer.name.toLowerCase().trim()
  );
  if (existingNameServer) {
    errorMessage.value = `服务器名称 "${newServer.name}" 已存在，请使用不同的名称！`;
    return;
  }
  
  // 验证服务器地址是否重复
  const serverAddress = `${newServer.host.trim()}:${newServer.port}`;
  const existingAddressServer = servers.value.find(server => 
    `${server.host}:${server.port}` === serverAddress
  );
  if (existingAddressServer) {
    errorMessage.value = `服务器地址 "${serverAddress}" 已存在（服务器：${existingAddressServer.name}），请使用不同的地址或端口！`;
    return;
  }
  
  // 验证端口范围
  if (newServer.port < 1024 || newServer.port > 65535) {
    errorMessage.value = '端口号必须在 1024-65535 范围内！';
    return;
  }
  
  // 验证主机地址格式（简单验证）
  const hostPattern = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$|^localhost$|^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$/;
  if (!hostPattern.test(newServer.host.trim())) {
    errorMessage.value = '请输入有效的主机地址（IP地址、localhost 或域名）！';
    return;
  }
  
  isLoading.value = true;
  try {
    // 调用后端添加服务器配置
    const result = await invoke('add_server_config', {
      name: newServer.name.trim(),
      host: newServer.host.trim(),
      port: newServer.port,
      accessToken: newServer.accessToken ? newServer.accessToken.trim() : null
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
    
    // 关闭弹窗并重置表单
    closeDialog();
    
  } catch (error) {
    console.error('添加服务器失败:', error);
    errorMessage.value = '添加服务器失败: ' + error;
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
/* 页面容器 */
.server-list-page {
  padding: 32px;
  background-color: #f5f5f1;
  min-height: 100vh;
}

/* 页面头部 */
.page-header {
  background: #fffcf6;
  border-radius: 15px;
  border: 1px solid #e4ddd3;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  padding: 32px;
  margin-bottom: 32px;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
}

.title-section {
  flex: 1;
}

.page-title {
  font-size: 32px;
  font-weight: 700;
  color: #4a593d;
  margin: 0 0 8px 0;
  letter-spacing: -0.5px;
}

.page-subtitle {
  font-size: 16px;
  color: #6e8b67;
  margin: 0;
  opacity: 0.8;
}

.header-stats {
  display: flex;
  gap: 16px;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 12px;
  background: linear-gradient(135deg, #f8f6f0 0%, #fffcf6 100%);
  padding: 16px 20px;
  border-radius: 15px;
  border: 1px solid #e4ddd3;
  min-width: 120px;
}

.stat-icon {
  font-size: 24px;
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.1));
}

.stat-number {
  font-size: 24px;
  font-weight: 700;
  color: #4a593d;
  line-height: 1;
}

.stat-label {
  font-size: 12px;
  color: #6e8b67;
  opacity: 0.8;
}
.header-actions {
  display: flex;
  align-items: center;
  gap: 20px;
}

.config-info {
  display: flex;
  align-items: center;
  gap: 8px;
  background: #f8f6f0;
  padding: 8px 16px;
  border-radius: 30px;
  border: 1px solid #e4ddd3;
}

.config-icon {
  font-size: 14px;
}

.config-text {
  font-size: 12px;
  color: #6e8b67;
  max-width: 200px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 按钮样式 */
.btn-primary {
  display: flex;
  align-items: center;
  gap: 8px;
  background: linear-gradient(135deg, #a9c3a6 0%, #8fb58b 100%);
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 30px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 4px 16px rgba(169, 195, 166, 0.3);
  letter-spacing: 0.3px;
}

.btn-primary:hover {
  background: linear-gradient(135deg, #8fb58b 0%, #7a9e76 100%);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(169, 195, 166, 0.4);
}

.btn-primary.btn-large {
  padding: 16px 32px;
  font-size: 16px;
}

.btn-secondary {
  display: flex;
  align-items: center;
  gap: 8px;
  background: #f8f6f0;
  color: #6e8b67;
  border: 1px solid #e4ddd3;
  padding: 12px 24px;
  border-radius: 30px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-secondary:hover {
  background: #f0ede6;
  border-color: #d4c7b8;
  transform: translateY(-1px);
}

.btn-icon {
  font-size: 16px;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.1));
}

.btn-text {
  font-weight: inherit;
}

/* 服务器内容区域 */
.servers-content {
  background: #fffcf6;
  border-radius: 15px;
  border: 1px solid #e4ddd3;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  overflow: hidden;
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 80px 40px;
}

.empty-illustration {
  position: relative;
  display: inline-block;
  margin-bottom: 32px;
}

.empty-icon {
  font-size: 80px;
  filter: drop-shadow(0 4px 12px rgba(0, 0, 0, 0.1));
  position: relative;
  z-index: 2;
}

.empty-decoration {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 120px;
  height: 120px;
  background: linear-gradient(135deg, #a9c3a6 0%, #8fb58b 100%);
  border-radius: 50%;
  opacity: 0.1;
  z-index: 1;
}

.empty-title {
  font-size: 24px;
  font-weight: 600;
  color: #4a593d;
  margin: 0 0 12px 0;
}

.empty-description {
  font-size: 16px;
  color: #6e8b67;
  margin: 0 0 32px 0;
  opacity: 0.8;
  max-width: 400px;
  margin-left: auto;
  margin-right: auto;
  line-height: 1.5;
}

/* 服务器网格 */
.server-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
  gap: 24px;
  padding: 32px;
}

/* 服务器卡片 */
.server-card {
  background: #fffcf6;
  border: 1px solid #e4ddd3;
  border-radius: 15px;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
}

.server-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
  border-color: #d4c7b8;
}

.server-card.server-connected {
  border-color: #a9c3a6;
  box-shadow: 0 2px 12px rgba(169, 195, 166, 0.2);
}

.server-card.server-connected:hover {
  box-shadow: 0 8px 32px rgba(169, 195, 166, 0.3);
}

/* 卡片头部 */
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 24px 24px 16px;
  border-bottom: 1px solid #f0ede6;
}

.server-title {
  flex: 1;
}

.server-name {
  font-size: 20px;
  font-weight: 600;
  color: #4a593d;
  margin: 0 0 8px 0;
  line-height: 1.2;
}

.server-address {
  font-size: 14px;
  color: #6e8b67;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  background: #f8f6f0;
  padding: 4px 8px;
  border-radius: 6px;
  display: inline-block;
}

/* 状态徽章 */
.status-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
  border: 1px solid;
}

.status-badge.status-connected {
  background: linear-gradient(135deg, #d4edda 0%, #c3e6cb 100%);
  color: #155724;
  border-color: #c3e6cb;
}

.status-badge.status-connecting {
  background: linear-gradient(135deg, #fff3cd 0%, #ffeaa7 100%);
  color: #856404;
  border-color: #ffeaa7;
}

.status-badge.status-disconnected {
  background: linear-gradient(135deg, #f8d7da 0%, #f5c6cb 100%);
  color: #721c24;
  border-color: #f5c6cb;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  animation: pulse 2s infinite;
}

.status-connected .status-dot {
  background: #28a745;
}

.status-connecting .status-dot {
  background: #ffc107;
}

.status-disconnected .status-dot {
  background: #dc3545;
}

@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.5; }
  100% { opacity: 1; }
}

/* 卡片内容 */
.card-content {
  padding: 0 24px 16px;
}

.server-metrics {
  display: flex;
  gap: 16px;
}

.metric-item {
  display: flex;
  align-items: center;
  gap: 8px;
  background: #f8f6f0;
  padding: 12px 16px;
  border-radius: 12px;
  flex: 1;
}

.metric-icon {
  font-size: 16px;
}

.metric-label {
  font-size: 12px;
  color: #6e8b67;
  flex: 1;
}

.metric-value {
  font-size: 14px;
  font-weight: 600;
  color: #4a593d;
}

/* 卡片操作 */
.card-actions {
  display: flex;
  gap: 12px;
  padding: 16px 24px 24px;
}

.btn-action {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  padding: 12px 16px;
  border: none;
  border-radius: 30px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
  justify-content: center;
}

.btn-action.btn-start {
  background: linear-gradient(135deg, #a9c3a6 0%, #8fb58b 100%);
  color: white;
  box-shadow: 0 2px 8px rgba(169, 195, 166, 0.3);
}

.btn-action.btn-start:hover {
  background: linear-gradient(135deg, #8fb58b 0%, #7a9e76 100%);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(169, 195, 166, 0.4);
}

.btn-action.btn-stop {
  background: linear-gradient(135deg, #f8d7da 0%, #f5c6cb 100%);
  color: #721c24;
  border: 1px solid #f5c6cb;
}

.btn-action.btn-stop:hover {
  background: linear-gradient(135deg, #f5c6cb 0%, #f1b0b7 100%);
  transform: translateY(-1px);
}

.btn-action:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none !important;
}

.btn-danger {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  background: #f8f6f0;
  color: #dc3545;
  border: 1px solid #e4ddd3;
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-danger:hover {
  background: #f8d7da;
  border-color: #f5c6cb;
  transform: translateY(-1px);
}

/* 弹窗样式 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(74, 89, 61, 0.6);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.modal-container {
  background: #fffcf6;
  border-radius: 15px;
  border: 1px solid #e4ddd3;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
  width: 90%;
  max-width: 600px;
  max-height: 90vh;
  overflow-y: auto;
  animation: slideUp 0.3s ease;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 32px 32px 24px;
  border-bottom: 1px solid #f0ede6;
}

.modal-title-section {
  flex: 1;
}

.modal-title {
  font-size: 28px;
  font-weight: 700;
  color: #4a593d;
  margin: 0 0 8px 0;
  line-height: 1.2;
}

.modal-subtitle {
  font-size: 16px;
  color: #6e8b67;
  margin: 0;
  opacity: 0.8;
}

.btn-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: #f8f6f0;
  border: 1px solid #e4ddd3;
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.3s ease;
  color: #6e8b67;
}

.btn-close:hover {
  background: #f0ede6;
  border-color: #d4c7b8;
  transform: scale(1.05);
}

.close-icon {
  font-size: 18px;
  font-weight: 600;
}

/* 警告样式 */
.alert {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px 20px;
  border-radius: 12px;
  margin: 0 32px 24px;
}

.alert-error {
  background: linear-gradient(135deg, #f8d7da 0%, #f5c6cb 100%);
  border: 1px solid #f5c6cb;
}

.alert-icon {
  font-size: 20px;
  flex-shrink: 0;
  margin-top: 2px;
}

.alert-content {
  flex: 1;
}

.alert-title {
  font-size: 14px;
  font-weight: 600;
  color: #721c24;
  margin: 0 0 4px 0;
}

.alert-message {
  font-size: 13px;
  color: #721c24;
  margin: 0;
  opacity: 0.9;
}

/* 表单样式 */
.modal-form {
  padding: 0 32px 32px;
}

.form-section {
  margin-bottom: 32px;
}

.form-section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-size: 18px;
  font-weight: 600;
  color: #4a593d;
  margin: 0 0 16px 0;
}

.form-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field-label {
  font-size: 14px;
  font-weight: 500;
  color: #4a593d;
}

.field-input {
  padding: 14px 16px;
  border: 1px solid #e4ddd3;
  border-radius: 15px;
  font-size: 14px;
  background: #fffcf6;
  color: #4a593d;
  transition: all 0.3s ease;
}

.field-input:focus {
  outline: none;
  border-color: #a9c3a6;
  box-shadow: 0 0 0 3px rgba(169, 195, 166, 0.1);
}

.field-hint {
  font-size: 12px;
  color: #6e8b67;
  opacity: 0.8;
}

.modal-actions {
  display: flex;
  gap: 16px;
  justify-content: flex-end;
  padding: 24px 32px 32px;
  border-top: 1px solid #f0ede6;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .server-list-page {
    padding: 16px;
  }

  .page-header {
    padding: 24px;
  }

  .header-content {
    flex-direction: column;
    gap: 20px;
  }

  .header-stats {
    width: 100%;
    justify-content: space-between;
  }

  .server-grid {
    grid-template-columns: 1fr;
    padding: 20px;
  }

  .form-grid {
    grid-template-columns: 1fr;
  }

  .modal-container {
    width: 95%;
    margin: 20px;
  }

  .modal-header,
  .modal-form,
  .modal-actions {
    padding-left: 20px;
    padding-right: 20px;
  }

  .modal-actions {
    flex-direction: column-reverse;
  }
}
</style> 