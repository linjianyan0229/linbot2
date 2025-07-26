<template>
  <div class="logs-container">
    <!-- 页面标题和控制面板 -->
    <div class="logs-header">
      <h2 class="section-title">实时日志</h2>
      <div class="logs-controls">
        <div class="control-group">
          <label class="control-label">
            <input 
              type="checkbox" 
              v-model="settings.show_heartbeat_logs"
              @change="updateSettings"
              class="control-checkbox"
            />
            显示心跳包
          </label>
        </div>
        <div class="control-group">
          <label class="control-label">
            <input 
              type="checkbox" 
              v-model="settings.auto_scroll_logs"
              @change="updateSettings"
              class="control-checkbox"
            />
            自动滚动
          </label>
        </div>
        <button @click="clearLogs" class="btn-secondary">清空日志</button>
      </div>
    </div>

    <!-- 日志显示区域 -->
    <div class="logs-content">
      <div 
        ref="logsContainer" 
        class="logs-list"
        :class="{ 'auto-scroll': settings.auto_scroll_logs }"
      >
        <div 
          v-for="logEntry in logs" 
          :key="logEntry.id"
          class="log-entry"
          :class="getLogEntryClass(logEntry)"
        >
          <div class="log-timestamp">
            {{ formatTimestamp(logEntry.timestamp) }}
          </div>
          <div class="log-level">
            <span class="level-badge" :class="`level-${logEntry.level}`">
              {{ logEntry.level.toUpperCase() }}
            </span>
          </div>
          <div class="log-content">
            <div class="log-message">{{ logEntry.content }}</div>
            <div v-if="logEntry.message_type" class="log-metadata">
              <span v-if="logEntry.message_type === 'group'" class="metadata-item group">
                群聊
              </span>
              <span v-else-if="logEntry.message_type === 'private'" class="metadata-item private">
                私聊
              </span>
              <span v-if="logEntry.group_id" class="metadata-item">
                群ID: {{ logEntry.group_id }}
              </span>
              <span v-if="logEntry.sender_name" class="metadata-item">
                {{ logEntry.sender_name }}
              </span>
            </div>
          </div>
        </div>
        
        <!-- 空状态提示 -->
        <div v-if="logs.length === 0" class="empty-state">
          <div class="empty-icon">📝</div>
          <div class="empty-text">暂无日志记录</div>
          <div class="empty-hint">启动服务器后将会显示实时日志</div>
        </div>
      </div>
    </div>
    
    <!-- 日志统计信息 -->
    <div class="logs-footer">
      <div class="log-stats">
        <span class="stat-item">总计: {{ logs.length }}</span>
        <span class="stat-item">消息: {{ messageCount }}</span>
        <span class="stat-item">心跳: {{ heartbeatCount }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// 响应式数据
const logs = ref([]);
const settings = reactive({
  show_heartbeat_logs: false,
  auto_scroll_logs: true,
  max_log_entries: 1000,
  log_buffer_size: 100,
});

const logsContainer = ref(null);
let unlistenLogEntry = null;

// 计算属性
const messageCount = computed(() => 
  logs.value.filter(log => log.category === 'message').length
);

const heartbeatCount = computed(() => 
  logs.value.filter(log => log.category === 'heartbeat').length
);

// 方法
const loadSettings = async () => {
  try {
    const appSettings = await invoke('get_app_settings');
    Object.assign(settings, appSettings);
  } catch (error) {
    console.error('加载设置失败:', error);
  }
};

const updateSettings = async () => {
  try {
    await invoke('update_app_settings', { settings });
    console.log('设置已更新');
  } catch (error) {
    console.error('更新设置失败:', error);
  }
};

const loadLogHistory = async () => {
  try {
    const history = await invoke('get_log_history');
    logs.value = history;
    await nextTick();
    scrollToBottom();
  } catch (error) {
    console.error('加载日志历史失败:', error);
  }
};

const clearLogs = async () => {
  try {
    await invoke('clear_log_history');
    logs.value = [];
    console.log('日志已清空');
  } catch (error) {
    console.error('清空日志失败:', error);
  }
};

const subscribeToLogs = async () => {
  try {
    // 订阅实时日志
    await invoke('subscribe_logs');
    
    // 监听日志事件
    unlistenLogEntry = await listen('log-entry', (event) => {
      const logEntry = event.payload;
      
      // 检查是否应该显示心跳包日志
      if (logEntry.category === 'heartbeat' && !settings.show_heartbeat_logs) {
        return;
      }
      
      // 添加到日志列表
      logs.value.push(logEntry);
      
      // 限制日志条目数量
      if (logs.value.length > settings.max_log_entries) {
        logs.value.splice(0, logs.value.length - settings.max_log_entries);
      }
      
      // 自动滚动到底部
      if (settings.auto_scroll_logs) {
        nextTick(() => scrollToBottom());
      }
    });
  } catch (error) {
    console.error('订阅日志失败:', error);
  }
};

const scrollToBottom = () => {
  if (logsContainer.value) {
    logsContainer.value.scrollTop = logsContainer.value.scrollHeight;
  }
};

const formatTimestamp = (timestamp) => {
  const date = new Date(timestamp);
  return date.toLocaleTimeString('zh-CN', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    timeZoneName: 'short'
  });
};

const getLogEntryClass = (logEntry) => {
  const classes = [`category-${logEntry.category}`, `level-${logEntry.level}`];
  
  if (logEntry.message_type === 'group') {
    classes.push('message-group');
  } else if (logEntry.message_type === 'private') {
    classes.push('message-private');
  }
  
  return classes;
};

// 生命周期
onMounted(async () => {
  await loadSettings();
  await loadLogHistory();
  await subscribeToLogs();
});

onUnmounted(() => {
  if (unlistenLogEntry) {
    unlistenLogEntry();
  }
});
</script>

<style scoped>
.logs-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--bg-color);
  color: var(--text-color);
}

.logs-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color);
  background-color: var(--card-bg);
}

.section-title {
  color: var(--title-color);
  font-size: 24px;
  font-weight: 600;
  margin: 0;
}

.logs-controls {
  display: flex;
  align-items: center;
  gap: 20px;
}

.control-group {
  display: flex;
  align-items: center;
}

.control-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: var(--text-color);
  cursor: pointer;
  user-select: none;
}

.control-checkbox {
  width: 16px;
  height: 16px;
  accent-color: var(--button-color);
}

.btn-secondary {
  padding: 8px 16px;
  background-color: var(--border-color);
  color: var(--text-color);
  border: none;
  border-radius: 15px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-secondary:hover {
  background-color: var(--button-hover-color);
  color: white;
}

.logs-content {
  flex: 1;
  overflow: hidden;
  background-color: var(--bg-color);
}

.logs-list {
  height: 100%;
  overflow-y: auto;
  padding: 16px 24px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
}

.log-entry {
  display: flex;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid rgba(228, 221, 211, 0.3);
  animation: logEntryFadeIn 0.3s ease-out;
}

@keyframes logEntryFadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.log-timestamp {
  min-width: 80px;
  color: #888;
  font-size: 11px;
  font-weight: 500;
}

.log-level {
  min-width: 60px;
}

.level-badge {
  display: inline-block;
  padding: 2px 6px;
  border-radius: 6px;
  font-size: 10px;
  font-weight: bold;
  text-transform: uppercase;
}

.level-debug {
  background-color: #e3f2fd;
  color: #1976d2;
}

.level-info {
  background-color: #e8f5e8;
  color: #2e7d32;
}

.level-warning {
  background-color: #fff3e0;
  color: #f57c00;
}

.level-error {
  background-color: #ffebee;
  color: #d32f2f;
}

.log-content {
  flex: 1;
  min-width: 0;
}

.log-message {
  word-wrap: break-word;
  margin-bottom: 4px;
}

.log-metadata {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.metadata-item {
  display: inline-block;
  padding: 2px 6px;
  background-color: rgba(169, 195, 166, 0.2);
  color: var(--text-color);
  border-radius: 4px;
  font-size: 11px;
}

.metadata-item.group {
  background-color: rgba(76, 175, 80, 0.2);
  color: #2e7d32;
}

.metadata-item.private {
  background-color: rgba(33, 150, 243, 0.2);
  color: #1565c0;
}

/* 不同类型日志的高亮 */
.category-message {
  background-color: rgba(169, 195, 166, 0.1);
}

.category-heartbeat {
  background-color: rgba(158, 158, 158, 0.05);
  color: #666;
}

.category-lifecycle {
  background-color: rgba(33, 150, 243, 0.1);
}

.category-notice {
  background-color: rgba(255, 193, 7, 0.1);
}

.category-request {
  background-color: rgba(156, 39, 176, 0.1);
}

.message-group .log-message {
  color: var(--title-color);
  font-weight: 500;
}

.message-private .log-message {
  color: #1565c0;
  font-weight: 500;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 300px;
  color: #999;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty-text {
  font-size: 18px;
  font-weight: 500;
  margin-bottom: 8px;
}

.empty-hint {
  font-size: 14px;
  color: #aaa;
}

.logs-footer {
  padding: 12px 24px;
  border-top: 1px solid var(--border-color);
  background-color: var(--card-bg);
}

.log-stats {
  display: flex;
  gap: 20px;
  font-size: 12px;
  color: #888;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

/* 滚动条样式 */
.logs-list::-webkit-scrollbar {
  width: 8px;
}

.logs-list::-webkit-scrollbar-track {
  background: var(--bg-color);
}

.logs-list::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 4px;
}

.logs-list::-webkit-scrollbar-thumb:hover {
  background: var(--button-hover-color);
}
</style> 