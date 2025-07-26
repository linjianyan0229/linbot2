<template>
  <div class="groups-container">
    <!-- 页面头部 -->
    <div class="groups-header">
      <h2 class="section-title">群聊列表</h2>
      <div class="header-controls">
        <!-- 机器人选择器（多服务器模式） -->
        <div v-if="showBotSelector" class="bot-selector">
          <select
            v-model="selectedBotId"
            @change="onBotChange"
            class="bot-select"
          >
            <option value="">选择机器人账号</option>
            <option
              v-for="bot in botAccounts"
              :key="bot.self_id"
              :value="bot.self_id"
            >
              {{ bot.nickname }} ({{ bot.self_id }})
            </option>
          </select>
        </div>

        <!-- 搜索框 -->
        <div class="search-box">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索群名称..."
            class="search-input"
          />
        </div>

        <!-- 刷新按钮 -->
        <button @click="refreshData" class="btn-refresh" :disabled="loading">
          <span class="refresh-icon">🔄</span>
          刷新
        </button>
      </div>
    </div>

    <!-- 内容区域 -->
    <div class="groups-content">
      <!-- 加载状态 -->
      <div v-if="loading" class="loading-state">
        <div class="loading-spinner"></div>
        <div class="loading-text">加载中...</div>
      </div>

      <!-- 错误状态 -->
      <div v-else-if="error" class="error-state">
        <div class="error-icon">⚠️</div>
        <div class="error-text">{{ error }}</div>
        <button @click="refreshData" class="btn-retry">重试</button>
      </div>

      <!-- 空状态 -->
      <div v-else-if="filteredGroups.length === 0 && !loading" class="empty-state">
        <div class="empty-icon">💬</div>
        <div class="empty-text">
          {{ selectedBotId ? '该机器人暂未加入任何群聊' : '请先选择机器人账号' }}
        </div>
        <div class="empty-hint">
          {{ selectedBotId ? '群聊列表为空或正在加载中' : '从上方下拉菜单中选择要查看的机器人账号' }}
        </div>
      </div>

      <!-- 群聊列表 -->
      <div v-else class="groups-list">
        <div
          v-for="group in paginatedGroups"
          :key="group.group_id"
          class="group-item"
        >
          <div class="group-avatar">
            <div class="avatar-placeholder">
              {{ group.group_name.charAt(0) }}
            </div>
          </div>
          <div class="group-info">
            <div class="group-name">{{ group.group_name }}</div>
            <div class="group-members">
              成员: {{ group.member_count }} / {{ group.max_member_count }}
            </div>
            <div class="group-id">群ID: {{ group.group_id }}</div>
          </div>
          <div class="group-stats">
            <div class="member-progress">
              <div class="progress-bar">
                <div
                  class="progress-fill"
                  :style="{ width: (group.member_count / group.max_member_count * 100) + '%' }"
                ></div>
              </div>
              <div class="progress-text">
                {{ Math.round(group.member_count / group.max_member_count * 100) }}%
              </div>
            </div>
          </div>
          <div class="group-actions">
            <button class="btn-action">发消息</button>
          </div>
        </div>
      </div>
    </div>

    <!-- 分页控制 -->
    <div v-if="totalPages > 1" class="pagination">
      <button
        @click="currentPage--"
        :disabled="currentPage <= 1"
        class="btn-page"
      >
        上一页
      </button>
      <span class="page-info">
        第 {{ currentPage }} 页，共 {{ totalPages }} 页
      </span>
      <button
        @click="currentPage++"
        :disabled="currentPage >= totalPages"
        class="btn-page"
      >
        下一页
      </button>
    </div>

    <!-- 状态栏 -->
    <div class="groups-footer">
      <div class="status-info">
        <span class="status-item">总计: {{ filteredGroups.length }} 个群聊</span>
        <span v-if="searchQuery" class="status-item">搜索结果: {{ filteredGroups.length }} 个</span>
        <span class="status-item">
          总成员: {{ totalMembers }} 人
        </span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// 响应式数据
const botAccounts = ref([]);
const groups = ref([]);
const selectedBotId = ref('');
const searchQuery = ref('');
const loading = ref(false);
const error = ref('');
const currentPage = ref(1);
const pageSize = 20;

// 计算属性
const showBotSelector = computed(() => botAccounts.value.length > 1);

const filteredGroups = computed(() => {
  if (!searchQuery.value) return groups.value;

  const query = searchQuery.value.toLowerCase();
  return groups.value.filter(group =>
    group.group_name.toLowerCase().includes(query) ||
    group.group_id.toString().includes(query)
  );
});

const totalPages = computed(() =>
  Math.ceil(filteredGroups.value.length / pageSize)
);

const paginatedGroups = computed(() => {
  const start = (currentPage.value - 1) * pageSize;
  const end = start + pageSize;
  return filteredGroups.value.slice(start, end);
});

const totalMembers = computed(() =>
  filteredGroups.value.reduce((sum, group) => sum + group.member_count, 0)
);

// 方法
const loadBotAccounts = async () => {
  try {
    const accounts = await invoke('get_bot_accounts');
    botAccounts.value = accounts;

    // 如果只有一个机器人，自动选择
    if (accounts.length === 1) {
      selectedBotId.value = accounts[0].self_id;
      await loadGroups();
    }
  } catch (err) {
    console.error('加载机器人账号失败:', err);
    error.value = '加载机器人账号失败: ' + err;
  }
};

const loadGroups = async () => {
  if (!selectedBotId.value) {
    groups.value = [];
    return;
  }

  loading.value = true;
  error.value = '';

  try {
    const groupList = await invoke('get_groups', {
      selfId: parseInt(selectedBotId.value)
    });
    groups.value = groupList;
    currentPage.value = 1; // 重置到第一页
  } catch (err) {
    console.error('加载群聊列表失败:', err);
    error.value = '加载群聊列表失败: ' + err;
    groups.value = [];
  } finally {
    loading.value = false;
  }
};

const refreshData = async () => {
  if (selectedBotId.value) {
    // 清除缓存
    try {
      await invoke('refresh_bot_data', {
        selfId: parseInt(selectedBotId.value)
      });
    } catch (err) {
      console.warn('清除缓存失败:', err);
    }
  }

  await loadBotAccounts();
  if (selectedBotId.value) {
    await loadGroups();
  }
};

const onBotChange = async () => {
  await loadGroups();
};

// 监听搜索查询变化，重置分页
watch(searchQuery, () => {
  currentPage.value = 1;
});

// 生命周期
onMounted(async () => {
  await loadBotAccounts();
});
</script>

<style scoped>
.groups-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--bg-color);
}

.groups-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color);
  background-color: var(--card-bg);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.section-title {
  color: var(--text-title);
  font-size: 24px;
  font-weight: 600;
  margin: 0;
}

.header-controls {
  display: flex;
  align-items: center;
  gap: 16px;
}

.bot-selector {
  min-width: 200px;
}

.bot-select {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 15px;
  background-color: var(--card-bg);
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.bot-select:focus {
  outline: none;
  border-color: var(--button-bg);
  box-shadow: 0 0 0 2px rgba(169, 195, 166, 0.2);
}

.search-box {
  min-width: 200px;
}

.search-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 15px;
  background-color: var(--card-bg);
  color: var(--text-primary);
  font-size: 14px;
  transition: all 0.2s ease;
}

.search-input:focus {
  outline: none;
  border-color: var(--button-bg);
  box-shadow: 0 0 0 2px rgba(169, 195, 166, 0.2);
}

.search-input::placeholder {
  color: #999;
}

.btn-refresh {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background-color: var(--button-bg);
  color: white;
  border: none;
  border-radius: 30px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-refresh:hover:not(:disabled) {
  background-color: var(--button-hover);
  transform: translateY(-1px);
}

.btn-refresh:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.refresh-icon {
  font-size: 12px;
}

.groups-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* 状态样式 */
.loading-state,
.error-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  padding: 60px 40px;
  text-align: center;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--border-color);
  border-top: 3px solid var(--button-bg);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 16px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.loading-text,
.error-text,
.empty-text {
  font-size: 18px;
  color: var(--text-title);
  font-weight: 500;
  margin-bottom: 8px;
}

.error-icon,
.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty-hint {
  font-size: 14px;
  color: #888;
  line-height: 1.5;
}

.btn-retry {
  margin-top: 16px;
  padding: 8px 16px;
  background-color: var(--button-bg);
  color: white;
  border: none;
  border-radius: 30px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-retry:hover {
  background-color: var(--button-hover);
}

/* 群聊列表样式 */
.groups-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px 24px;
}

.group-item {
  display: flex;
  align-items: center;
  padding: 16px;
  margin-bottom: 12px;
  background-color: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 15px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  transition: all 0.2s ease;
}

.group-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
}

.group-avatar {
  margin-right: 16px;
}

.avatar-placeholder {
  width: 48px;
  height: 48px;
  border-radius: 15px;
  background: linear-gradient(135deg, var(--button-bg), var(--button-hover));
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 18px;
  font-weight: 600;
}

.group-info {
  flex: 1;
  min-width: 0;
  margin-right: 16px;
}

.group-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-title);
  margin-bottom: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.group-members {
  font-size: 14px;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.group-id {
  font-size: 12px;
  color: #888;
}

.group-stats {
  min-width: 120px;
  margin-right: 16px;
}

.member-progress {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.progress-bar {
  width: 100%;
  height: 6px;
  background-color: var(--border-color);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--button-bg), var(--button-hover));
  border-radius: 3px;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 11px;
  color: #888;
  text-align: center;
}

.group-actions {
  margin-left: 16px;
}

.btn-action {
  padding: 6px 12px;
  background-color: var(--border-color);
  color: var(--text-primary);
  border: none;
  border-radius: 15px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-action:hover {
  background-color: var(--button-bg);
  color: white;
}

/* 分页样式 */
.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 16px 24px;
  border-top: 1px solid var(--border-color);
  background-color: var(--card-bg);
}

.btn-page {
  padding: 8px 16px;
  background-color: var(--button-bg);
  color: white;
  border: none;
  border-radius: 30px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-page:hover:not(:disabled) {
  background-color: var(--button-hover);
}

.btn-page:disabled {
  background-color: var(--border-color);
  color: #888;
  cursor: not-allowed;
}

.page-info {
  font-size: 14px;
  color: var(--text-primary);
}

/* 状态栏样式 */
.groups-footer {
  padding: 12px 24px;
  border-top: 1px solid var(--border-color);
  background-color: var(--card-bg);
}

.status-info {
  display: flex;
  gap: 20px;
  font-size: 12px;
  color: #888;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

/* 滚动条样式 */
.groups-list::-webkit-scrollbar {
  width: 6px;
}

.groups-list::-webkit-scrollbar-track {
  background: var(--border-color);
  border-radius: 3px;
}

.groups-list::-webkit-scrollbar-thumb {
  background: var(--button-bg);
  border-radius: 3px;
}

.groups-list::-webkit-scrollbar-thumb:hover {
  background: var(--button-hover);
}
</style>