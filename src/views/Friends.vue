<template>
  <div class="friends-container">
    <!-- 页面头部 -->
    <div class="friends-header">
      <h2 class="section-title">好友列表</h2>
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
            placeholder="搜索好友昵称..."
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
    <div class="friends-content">
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
      <div v-else-if="filteredFriends.length === 0 && !loading" class="empty-state">
        <div class="empty-icon">👥</div>
        <div class="empty-text">
          {{ selectedBotId ? '该机器人暂无好友' : '请先选择机器人账号' }}
        </div>
        <div class="empty-hint">
          {{ selectedBotId ? '好友列表为空或正在加载中' : '从上方下拉菜单中选择要查看的机器人账号' }}
        </div>
      </div>

      <!-- 好友列表 -->
      <div v-else class="friends-list">
        <div
          v-for="friend in paginatedFriends"
          :key="friend.user_id"
          class="friend-item"
        >
          <div class="friend-avatar">
            <div class="avatar-placeholder">
              {{ friend.nickname.charAt(0) }}
            </div>
          </div>
          <div class="friend-info">
            <div class="friend-name">{{ friend.nickname }}</div>
            <div class="friend-remark" v-if="friend.remark && friend.remark !== friend.nickname">
              备注: {{ friend.remark }}
            </div>
            <div class="friend-id">ID: {{ friend.user_id }}</div>
          </div>
          <div class="friend-actions">
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
    <div class="friends-footer">
      <div class="status-info">
        <span class="status-item">总计: {{ filteredFriends.length }} 个好友</span>
        <span v-if="searchQuery" class="status-item">搜索结果: {{ filteredFriends.length }} 个</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// 响应式数据
const botAccounts = ref([]);
const friends = ref([]);
const selectedBotId = ref('');
const searchQuery = ref('');
const loading = ref(false);
const error = ref('');
const currentPage = ref(1);
const pageSize = 20;

// 计算属性
const showBotSelector = computed(() => botAccounts.value.length > 1);

const filteredFriends = computed(() => {
  if (!searchQuery.value) return friends.value;

  const query = searchQuery.value.toLowerCase();
  return friends.value.filter(friend =>
    friend.nickname.toLowerCase().includes(query) ||
    friend.remark.toLowerCase().includes(query) ||
    friend.user_id.toString().includes(query)
  );
});

const totalPages = computed(() =>
  Math.ceil(filteredFriends.value.length / pageSize)
);

const paginatedFriends = computed(() => {
  const start = (currentPage.value - 1) * pageSize;
  const end = start + pageSize;
  return filteredFriends.value.slice(start, end);
});

// 方法
const loadBotAccounts = async () => {
  try {
    const accounts = await invoke('get_bot_accounts');
    botAccounts.value = accounts;

    // 如果只有一个机器人，自动选择
    if (accounts.length === 1) {
      selectedBotId.value = accounts[0].self_id;
      await loadFriends();
    }
  } catch (err) {
    console.error('加载机器人账号失败:', err);
    error.value = '加载机器人账号失败: ' + err;
  }
};

const loadFriends = async () => {
  if (!selectedBotId.value) {
    friends.value = [];
    return;
  }

  loading.value = true;
  error.value = '';

  try {
    const friendList = await invoke('get_friends', {
      selfId: parseInt(selectedBotId.value)
    });
    friends.value = friendList;
    currentPage.value = 1; // 重置到第一页
  } catch (err) {
    console.error('加载好友列表失败:', err);
    error.value = '加载好友列表失败: ' + err;
    friends.value = [];
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
    await loadFriends();
  }
};

const onBotChange = async () => {
  await loadFriends();
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
.friends-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--bg-color);
}

.friends-header {
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

.friends-content {
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

/* 好友列表样式 */
.friends-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px 24px;
}

.friend-item {
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

.friend-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
}

.friend-avatar {
  margin-right: 16px;
}

.avatar-placeholder {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--button-bg), var(--button-hover));
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 18px;
  font-weight: 600;
}

.friend-info {
  flex: 1;
  min-width: 0;
}

.friend-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-title);
  margin-bottom: 4px;
}

.friend-remark {
  font-size: 14px;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.friend-id {
  font-size: 12px;
  color: #888;
}

.friend-actions {
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
.friends-footer {
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
.friends-list::-webkit-scrollbar {
  width: 6px;
}

.friends-list::-webkit-scrollbar-track {
  background: var(--border-color);
  border-radius: 3px;
}

.friends-list::-webkit-scrollbar-thumb {
  background: var(--button-bg);
  border-radius: 3px;
}

.friends-list::-webkit-scrollbar-thumb:hover {
  background: var(--button-hover);
}
</style>