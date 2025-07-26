<template>
  <div class="logs-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <div class="title-section">
          <h1 class="page-title">实时日志</h1>
          <p class="page-subtitle">监控系统运行状态和消息记录 • 超过4条日志时自动滚动</p>
        </div>
        <div class="header-stats">
          <div class="stat-card">
            <div class="stat-icon">📝</div>
            <div class="stat-info">
              <div class="stat-number">{{ logs.length }}</div>
              <div class="stat-label">总日志数</div>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-icon">💬</div>
            <div class="stat-info">
              <div class="stat-number">{{ messageCount }}</div>
              <div class="stat-label">消息记录</div>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-icon">💓</div>
            <div class="stat-info">
              <div class="stat-number">{{ heartbeatCount }}</div>
              <div class="stat-label">心跳包</div>
            </div>
          </div>
        </div>
      </div>

      <div class="header-controls">
        <div class="control-group">
          <div class="toggle-switch">
            <input
              type="checkbox"
              id="heartbeat-toggle"
              v-model="settings.show_heartbeat_logs"
              @change="updateSettings"
              class="toggle-input"
            />
            <label for="heartbeat-toggle" class="toggle-label">
              <span class="toggle-slider"></span>
              <span class="toggle-text">显示心跳包</span>
            </label>
          </div>
        </div>

        <div class="control-group">
          <div class="toggle-switch">
            <input
              type="checkbox"
              id="autoscroll-toggle"
              v-model="settings.auto_scroll_logs"
              @change="updateSettings"
              class="toggle-input"
            />
            <label for="autoscroll-toggle" class="toggle-label">
              <span class="toggle-slider"></span>
              <span class="toggle-text">
                自动滚动
                <span v-if="filteredLogs.length <= 4" class="toggle-hint">(需要>4条日志)</span>
                <span v-else-if="autoScrollEnabled" class="toggle-hint">(已启用)</span>
              </span>
            </label>
          </div>
        </div>

        <button @click="clearLogs" class="btn-secondary">
          <span class="btn-icon">🗑️</span>
          <span class="btn-text">清空日志</span>
        </button>
      </div>
    </div>

    <!-- 日志内容区域 -->
    <div class="logs-content">
      <div v-if="logs.length === 0" class="empty-state">
        <div class="empty-illustration">
          <div class="empty-icon">📝</div>
          <div class="empty-decoration"></div>
        </div>
        <h3 class="empty-title">暂无日志记录</h3>
        <p class="empty-description">启动服务器后将会显示实时日志信息，包括连接状态、消息记录等</p>
      </div>

      <div v-else class="logs-container">
        <div
          ref="logsContainer"
          class="logs-list"
          :class="{ 'auto-scroll': settings.auto_scroll_logs }"
        >
          <div
            v-for="logEntry in filteredLogs"
            :key="logEntry.id"
            class="log-entry"
            :class="getLogEntryClass(logEntry)"
          >
            <div class="log-meta">
              <div class="log-timestamp">
                {{ formatTimestamp(logEntry.timestamp) }}
              </div>
              <div class="log-level">
                <span class="level-badge" :class="`level-${logEntry.level}`">
                  {{ getLevelIcon(logEntry.level) }}
                  <span class="level-text">{{ logEntry.level.toUpperCase() }}</span>
                </span>
              </div>
            </div>

            <div class="log-content">
              <div class="log-message">{{ logEntry.content }}</div>
              <div v-if="logEntry.message_type || logEntry.sender_name" class="log-metadata">
                <span v-if="logEntry.message_type === 'group'" class="metadata-tag tag-group">
                  <span class="tag-icon">👥</span>
                  <span class="tag-text">群聊</span>
                </span>
                <span v-else-if="logEntry.message_type === 'private'" class="metadata-tag tag-private">
                  <span class="tag-icon">👤</span>
                  <span class="tag-text">私聊</span>
                </span>
                <span v-if="logEntry.group_id" class="metadata-tag tag-id">
                  <span class="tag-text">群ID: {{ logEntry.group_id }}</span>
                </span>
                <span v-if="logEntry.sender_name" class="metadata-tag tag-sender">
                  <span class="tag-text">{{ logEntry.sender_name }}</span>
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- 日志底部信息 -->
        <div class="logs-footer">
          <div class="footer-info">
            <span class="info-text">
              显示 {{ filteredLogs.length }} / {{ logs.length }} 条日志
            </span>
            <span v-if="!settings.show_heartbeat_logs && heartbeatCount > 0" class="info-text">
              (已隐藏 {{ heartbeatCount }} 条心跳包)
            </span>
          </div>
          <div class="footer-actions">
            <button @click="scrollToTop" class="btn-scroll" title="回到顶部">
              <span class="btn-icon">⬆️</span>
            </button>
            <button @click="scrollToBottom" class="btn-scroll" title="滚动到底部">
              <span class="btn-icon">⬇️</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, computed, watch, onMounted, onUnmounted, nextTick } from 'vue';
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

// 用于跟踪已显示的日志ID，防止重复
const displayedLogIds = ref(new Set());

// 计算属性
const messageCount = computed(() => 
  logs.value.filter(log => log.category === 'message').length
);

const heartbeatCount = computed(() =>
  logs.value.filter(log => log.category === 'heartbeat').length
);

// 自动滚动状态
const autoScrollEnabled = computed(() => {
  return filteredLogs.value.length > 4 && settings.auto_scroll_logs;
});

// 过滤后的日志列表
const filteredLogs = computed(() => {
  if (settings.show_heartbeat_logs) {
    return logs.value;
  }
  return logs.value.filter(log => log.category !== 'heartbeat');
});

// 监听过滤后的日志变化，自动滚动到底部
watch(filteredLogs, (newLogs, oldLogs) => {
  // 当日志数量从4条或以下增加到超过4条时，强制滚动到底部
  if (oldLogs && oldLogs.length <= 4 && newLogs.length > 4) {
    nextTick(() => scrollToBottom());
    return;
  }

  // 正常的自动滚动逻辑
  if (shouldAutoScroll()) {
    nextTick(() => scrollToBottom());
  }
}, { flush: 'post' });

// 工具方法
const addLogWithDeduplication = (logEntry) => {
  // 检查是否已存在相同ID的日志
  if (displayedLogIds.value.has(logEntry.id)) {
    return false; // 已存在，跳过
  }

  // 添加到显示列表和ID集合
  displayedLogIds.value.add(logEntry.id);
  logs.value.push(logEntry);

  // 限制日志条目数量
  if (logs.value.length > settings.max_log_entries) {
    const removedLog = logs.value.shift();
    if (removedLog) {
      displayedLogIds.value.delete(removedLog.id);
    }
  }

  return true; // 成功添加
};

const clearAllLogs = () => {
  logs.value = [];
  displayedLogIds.value.clear();
};

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
    // 清空现有日志，避免重复
    clearAllLogs();

    const history = await invoke('get_log_history');

    // 使用去重逻辑加载历史日志
    for (const logEntry of history) {
      addLogWithDeduplication(logEntry);
    }

    await nextTick();
    scrollToBottom();
  } catch (error) {
    console.error('加载日志历史失败:', error);
  }
};

const clearLogs = async () => {
  try {
    await invoke('clear_log_history');
    clearAllLogs();
    console.log('日志已清空');
  } catch (error) {
    console.error('清空日志失败:', error);
  }
};

const subscribeToLogs = async () => {
  try {
    // 如果已经有监听器，先清理
    if (unlistenLogEntry) {
      unlistenLogEntry();
      unlistenLogEntry = null;
    }

    // 订阅实时日志
    await invoke('subscribe_logs');

    // 监听日志事件
    unlistenLogEntry = await listen('log-entry', (event) => {
      const logEntry = event.payload;

      // 检查是否应该显示心跳包日志
      if (logEntry.category === 'heartbeat' && !settings.show_heartbeat_logs) {
        return;
      }

      // 使用去重机制添加日志
      const added = addLogWithDeduplication(logEntry);
      if (!added) {
        return; // 已存在，跳过
      }

      // 自动滚动到底部
      if (shouldAutoScroll()) {
        nextTick(() => scrollToBottom());
      }
    });
  } catch (error) {
    console.error('订阅日志失败:', error);
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

// 获取日志级别图标
const getLevelIcon = (level) => {
  switch (level.toLowerCase()) {
    case 'info': return 'ℹ️';
    case 'warn': return '⚠️';
    case 'error': return '❌';
    case 'debug': return '🐛';
    default: return '📝';
  }
};

// 滚动到顶部
const scrollToTop = () => {
  if (logsContainer.value) {
    logsContainer.value.scrollTop = 0;
  }
};

// 滚动到底部
const scrollToBottom = () => {
  if (logsContainer.value) {
    // 使用 requestAnimationFrame 确保 DOM 更新完成后再滚动
    requestAnimationFrame(() => {
      logsContainer.value.scrollTop = logsContainer.value.scrollHeight;
    });
  }
};

// 检查是否应该自动滚动
const shouldAutoScroll = () => {
  // 只有当日志数量超过4条时才启用自动滚动
  if (filteredLogs.value.length <= 4) return false;

  // 检查自动滚动开关是否开启
  if (!settings.auto_scroll_logs) return false;

  // 检查容器是否存在
  if (!logsContainer.value) return false;

  const { scrollTop, scrollHeight, clientHeight } = logsContainer.value;
  // 如果用户滚动到距离底部50px以内，认为应该自动滚动
  return scrollHeight - scrollTop - clientHeight < 50;
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
  // 清理事件监听器
  if (unlistenLogEntry) {
    unlistenLogEntry();
    unlistenLogEntry = null;
  }

  // 清理日志数据（可选，有助于内存管理）
  clearAllLogs();
});
</script>

<style scoped>
/* 页面容器 */
.logs-page {
  padding: 20px;
  background-color: #f5f5f1;
  height: 720px; /* 固定应用高度 */
  max-height: 720px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* 页面头部 */
.page-header {
  background: #fffcf6;
  border-radius: 15px;
  border: 1px solid #e4ddd3;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  padding: 20px;
  margin-bottom: 20px;
  flex-shrink: 0; /* 防止头部被压缩 */
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
  font-size: 24px;
  font-weight: 700;
  color: #4a593d;
  margin: 0 0 6px 0;
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
  gap: 10px;
  background: linear-gradient(135deg, #f8f6f0 0%, #fffcf6 100%);
  padding: 12px 16px;
  border-radius: 15px;
  border: 1px solid #e4ddd3;
  min-width: 100px;
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

.header-controls {
  display: flex;
  align-items: center;
  gap: 24px;
}

.control-group {
  display: flex;
  align-items: center;
}

/* 切换开关样式 */
.toggle-switch {
  display: flex;
  align-items: center;
  gap: 12px;
}

.toggle-input {
  display: none;
}

.toggle-label {
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  user-select: none;
}

.toggle-slider {
  position: relative;
  width: 48px;
  height: 24px;
  background: #e4ddd3;
  border-radius: 24px;
  transition: all 0.3s ease;
}

.toggle-slider::before {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 20px;
  height: 20px;
  background: white;
  border-radius: 50%;
  transition: all 0.3s ease;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.toggle-input:checked + .toggle-label .toggle-slider {
  background: #a9c3a6;
}

.toggle-input:checked + .toggle-label .toggle-slider::before {
  transform: translateX(24px);
}

.toggle-text {
  font-size: 14px;
  color: #4a593d;
  font-weight: 500;
}

.toggle-hint {
  font-size: 12px;
  color: #6e8b67;
  opacity: 0.7;
  font-weight: 400;
}

/* 按钮样式 */
.btn-secondary {
  display: flex;
  align-items: center;
  gap: 8px;
  background: #f8f6f0;
  color: #6e8b67;
  border: 1px solid #e4ddd3;
  padding: 12px 20px;
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
}

.btn-text {
  font-weight: inherit;
}

/* 日志内容区域 */
.logs-content {
  flex: 1;
  background: #fffcf6;
  border-radius: 15px;
  border: 1px solid #e4ddd3;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0; /* 允许收缩 */
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 80px 40px;
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
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
  margin: 0;
  opacity: 0.8;
  max-width: 400px;
  line-height: 1.5;
}

/* 日志容器 */
.logs-container {
  display: flex;
  flex-direction: column;
  flex: 1; /* 占用剩余空间 */
  min-height: 0; /* 允许收缩 */
}

.logs-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 13px;
  line-height: 1.4;
  border: 1px solid #e4ddd3;
  border-radius: 15px;
  background: #fffcf6;
}

/* 自定义滚动条样式 */
.logs-list::-webkit-scrollbar {
  width: 8px;
}

.logs-list::-webkit-scrollbar-track {
  background: #f5f5f1;
  border-radius: 4px;
}

.logs-list::-webkit-scrollbar-thumb {
  background: #a9c3a6;
  border-radius: 4px;
  transition: background 0.3s ease;
}

.logs-list::-webkit-scrollbar-thumb:hover {
  background: #8fb58b;
}

/* 日志条目 */
.log-entry {
  display: flex;
  gap: 12px;
  padding: 6px 12px;
  margin-bottom: 2px;
  background: #f8f6f0;
  border: 1px solid #f0ede6;
  border-radius: 8px;
  transition: all 0.2s ease;
  animation: logEntryFadeIn 0.3s ease-out;
}

.log-entry:hover {
  background: #f0ede6;
  border-color: #e4ddd3;
  transform: translateX(2px);
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

/* 日志元信息 */
.log-meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 140px;
}

.log-timestamp {
  font-size: 11px;
  color: #6e8b67;
  font-weight: 500;
  opacity: 0.8;
}

.log-level {
  display: flex;
  align-items: center;
}

.level-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 12px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  border: 1px solid;
}

.level-text {
  font-size: 10px;
}

/* 日志级别样式 */
.level-debug {
  background: linear-gradient(135deg, #e3f2fd 0%, #bbdefb 100%);
  color: #1976d2;
  border-color: #bbdefb;
}

.level-info {
  background: linear-gradient(135deg, #e8f5e8 0%, #c8e6c9 100%);
  color: #2e7d32;
  border-color: #c8e6c9;
}

.level-warn {
  background: linear-gradient(135deg, #fff3e0 0%, #ffcc02 100%);
  color: #f57c00;
  border-color: #ffcc02;
}

.level-error {
  background: linear-gradient(135deg, #ffebee 0%, #ffcdd2 100%);
  color: #d32f2f;
  border-color: #ffcdd2;
}

/* 日志内容 */
.log-content {
  flex: 1;
  min-width: 0;
}

.log-message {
  color: #4a593d;
  font-size: 13px;
  line-height: 1.5;
  word-wrap: break-word;
  margin-bottom: 8px;
}

.log-metadata {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.metadata-tag {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 500;
  border: 1px solid;
}

.tag-group {
  background: linear-gradient(135deg, #e8f5e8 0%, #c8e6c9 100%);
  color: #2e7d32;
  border-color: #c8e6c9;
}

.tag-private {
  background: linear-gradient(135deg, #e3f2fd 0%, #bbdefb 100%);
  color: #1976d2;
  border-color: #bbdefb;
}

.tag-id {
  background: linear-gradient(135deg, #f3e5f5 0%, #e1bee7 100%);
  color: #7b1fa2;
  border-color: #e1bee7;
}

.tag-sender {
  background: linear-gradient(135deg, #fff3e0 0%, #ffe0b2 100%);
  color: #f57c00;
  border-color: #ffe0b2;
}

.tag-icon {
  font-size: 10px;
}

.tag-text {
  font-size: 11px;
}

/* 不同类型日志的特殊样式 */
.category-message {
  border-left: 4px solid #a9c3a6;
}

.category-heartbeat {
  opacity: 0.7;
  border-left: 4px solid #e4ddd3;
}

.category-lifecycle {
  border-left: 4px solid #2196f3;
}

.category-notice {
  border-left: 4px solid #ff9800;
}

.category-request {
  border-left: 4px solid #9c27b0;
}

/* 日志底部 */
.logs-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  border-top: 1px solid #f0ede6;
  flex-shrink: 0; /* 防止底部被压缩 */
  background: #f8f6f0;
}

.footer-info {
  display: flex;
  gap: 16px;
  align-items: center;
}

.info-text {
  font-size: 12px;
  color: #6e8b67;
  opacity: 0.8;
}

.footer-actions {
  display: flex;
  gap: 8px;
}

.btn-scroll {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: #f0ede6;
  border: 1px solid #e4ddd3;
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.3s ease;
  color: #6e8b67;
}

.btn-scroll:hover {
  background: #e4ddd3;
  transform: translateY(-1px);
}

/* 滚动条样式 */
.logs-list::-webkit-scrollbar {
  width: 8px;
}

.logs-list::-webkit-scrollbar-track {
  background: #f8f6f0;
  border-radius: 4px;
}

.logs-list::-webkit-scrollbar-thumb {
  background: #e4ddd3;
  border-radius: 4px;
}

.logs-list::-webkit-scrollbar-thumb:hover {
  background: #d4c7b8;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .logs-page {
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

  .header-controls {
    flex-direction: column;
    gap: 16px;
    align-items: stretch;
  }

  .log-entry {
    flex-direction: column;
    gap: 8px;
  }

  .log-meta {
    flex-direction: row;
    justify-content: space-between;
    min-width: auto;
  }
}
</style> 