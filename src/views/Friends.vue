<template>
  <div class="friends-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <div class="title-section">
          <h1 class="page-title">好友列表</h1>
          <p class="page-subtitle">管理和查看机器人的好友信息</p>
        </div>

        <div class="header-stats">
          <div class="stat-card">
            <div class="stat-icon">👥</div>
            <div class="stat-info">
              <div class="stat-number">{{ filteredFriends.length }}</div>
              <div class="stat-label">好友总数</div>
            </div>
          </div>
          <div class="stat-card" v-if="searchQuery">
            <div class="stat-icon">🔍</div>
            <div class="stat-info">
              <div class="stat-number">{{ filteredFriends.length }}</div>
              <div class="stat-label">搜索结果</div>
            </div>
          </div>
        </div>
      </div>

      <div class="header-controls">
        <div class="control-group">
          <!-- 机器人选择器 -->
          <div v-if="showBotSelector" class="control-item">
            <label class="control-label">机器人账号</label>
            <select
              v-model="selectedBotId"
              @change="onBotChange"
              class="control-select"
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
          <div class="control-item">
            <label class="control-label">搜索好友</label>
            <input
              v-model="searchQuery"
              type="text"
              placeholder="输入昵称、备注或ID..."
              class="control-input"
            />
          </div>
        </div>

        <div class="action-buttons">
          <button @click="refreshData" class="btn-primary" :disabled="loading">
            <span class="btn-icon">🔄</span>
            <span class="btn-text">{{ loading ? '刷新中...' : '刷新数据' }}</span>
          </button>
        </div>
      </div>
    </div>

    <!-- 好友内容区域 -->
    <div class="friends-content">
      <!-- 空状态 -->
      <div v-if="friends.length === 0" class="empty-state">
        <div class="empty-illustration">
          <div class="empty-icon">👥</div>
          <div class="empty-decoration"></div>
        </div>
        <h3 class="empty-title">
          {{ selectedBotId ? '暂无好友' : '请选择机器人账号' }}
        </h3>
        <p class="empty-description">
          {{ selectedBotId ? '该机器人还没有添加任何好友，或好友列表正在加载中' : '请从上方选择要查看的机器人账号' }}
        </p>
      </div>

      <div v-else class="friends-container">
        <!-- 好友列表 -->
        <div
          ref="friendsContainer"
          class="friends-list"
        >
          <div
            v-for="friend in paginatedFriends"
            :key="friend.user_id"
            class="friend-entry"
          >
            <div class="friend-avatar">
              <img
                :src="getFriendAvatar(friend.user_id)"
                :alt="friend.nickname"
                class="avatar-image"
                @error="handleAvatarError"
              />
              <div class="avatar-status"></div>
            </div>

            <div class="friend-info">
              <div class="friend-name">{{ friend.nickname }}</div>
              <div class="friend-details">
                <span v-if="friend.remark && friend.remark !== friend.nickname" class="friend-remark">
                  备注: {{ friend.remark }}
                </span>
                <span class="friend-id">ID: {{ friend.user_id }}</span>
              </div>
            </div>

            <div class="friend-actions">
              <button @click="openMessageWindow(friend)" class="btn-message" title="发送消息">
                <span class="btn-icon">💬</span>
                <span class="btn-text">发消息</span>
              </button>
            </div>
          </div>
        </div>

        <!-- 分页和状态信息 -->
        <div class="friends-footer">
          <div class="footer-info">
            <span class="info-text">
              显示 {{ paginatedFriends.length }} / {{ filteredFriends.length }} 个好友
            </span>
            <span v-if="searchQuery" class="info-text">
              (搜索结果: {{ filteredFriends.length }} 个)
            </span>
          </div>
          <div class="footer-actions">
            <button
              @click="currentPage--"
              :disabled="currentPage <= 1"
              class="btn-page"
              title="上一页"
            >
              <span class="btn-icon">⬅️</span>
            </button>
            <span class="page-info">{{ currentPage }} / {{ totalPages }}</span>
            <button
              @click="currentPage++"
              :disabled="currentPage >= totalPages"
              class="btn-page"
              title="下一页"
            >
              <span class="btn-icon">➡️</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 消息窗口 -->
    <MessageWindow
      v-if="messageWindow.visible"
      :contact-type="'private'"
      :contact-id="messageWindow.contactId"
      :contact-name="messageWindow.contactName"
      :visible="messageWindow.visible"
      @close="closeMessageWindow"
      @message-sent="onMessageSent"
    />

    <!-- 加载遮罩 -->
    <div v-if="loading" class="loading-overlay">
      <div class="loading-content">
        <div class="loading-spinner"></div>
        <div class="loading-text">正在加载好友列表...</div>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="error-toast">
      <div class="error-content">
        <span class="error-icon">⚠️</span>
        <span class="error-message">{{ error }}</span>
        <button @click="error = ''" class="error-close">✕</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import MessageWindow from '../components/MessageWindow.vue';

// 响应式数据
const botAccounts = ref([]);
const friends = ref([]);
const selectedBotId = ref('');
const searchQuery = ref('');
const loading = ref(false);
const error = ref('');
const currentPage = ref(1);
const pageSize = 20;

// 消息窗口状态
const messageWindow = ref({
  visible: false,
  contactId: 0,
  contactName: ''
});

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

// 头像相关方法
const getFriendAvatar = (userId) => {
  return `https://q1.qlogo.cn/g?b=qq&nk=${userId}&s=640`;
};

const handleAvatarError = (event) => {
  // 头像加载失败时使用默认头像
  event.target.src = 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDAiIGhlaWdodD0iNDAiIHZpZXdCb3g9IjAgMCA0MCA0MCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPGNpcmNsZSBjeD0iMjAiIGN5PSIyMCIgcj0iMjAiIGZpbGw9IiNFNUU3RUIiLz4KPGNpcmNsZSBjeD0iMjAiIGN5PSIxNiIgcj0iNiIgZmlsbD0iIzlDQTNBRiIvPgo8cGF0aCBkPSJNMzAgMzJDMzAgMjYuNDc3MSAyNS41MjI5IDIyIDIwIDIyQzE0LjQ3NzEgMjIgMTAgMjYuNDc3MSAxMCAzMiIgZmlsbD0iIzlDQTNBRiIvPgo8L3N2Zz4K';
};

// 消息窗口相关方法
const openMessageWindow = (friend) => {
  messageWindow.value = {
    visible: true,
    contactId: friend.user_id,
    contactName: friend.nickname
  };
};

const closeMessageWindow = () => {
  messageWindow.value.visible = false;
};

const onMessageSent = (message) => {
  console.log('消息已发送:', message);
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
/* 页面容器 */
.friends-page {
  padding: 20px;
  background-color: #f5f5f1;
  height: 720px;
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
  flex-shrink: 0;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
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
  font-size: 20px;
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.1));
}

.stat-number {
  font-size: 20px;
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
  gap: 20px;
}

.control-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.control-label {
  font-size: 12px;
  font-weight: 600;
  color: #6e8b67;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.control-select,
.control-input {
  padding: 10px 14px;
  border: 1px solid #e4ddd3;
  border-radius: 15px;
  background: #fffcf6;
  color: #4a593d;
  font-size: 14px;
  min-width: 180px;
  transition: all 0.3s ease;
}

.control-select:focus,
.control-input:focus {
  outline: none;
  border-color: #a9c3a6;
  box-shadow: 0 0 0 3px rgba(169, 195, 166, 0.1);
  transform: translateY(-1px);
}

.control-input::placeholder {
  color: #a0a0a0;
}

.action-buttons {
  display: flex;
  gap: 12px;
}

.btn-primary {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  background: #a9c3a6;
  color: white;
  border: none;
  border-radius: 30px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 2px 8px rgba(169, 195, 166, 0.3);
}

.btn-primary:hover:not(:disabled) {
  background: #8fb58b;
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(169, 195, 166, 0.4);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.btn-icon {
  font-size: 14px;
}

.btn-text {
  font-weight: inherit;
}

/* 好友内容区域 */
.friends-content {
  flex: 1;
  background: #fffcf6;
  border-radius: 15px;
  border: 1px solid #e4ddd3;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 60px 40px;
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
}

.empty-illustration {
  position: relative;
  margin-bottom: 32px;
}

.empty-icon {
  font-size: 64px;
  opacity: 0.6;
  filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.1));
}

.empty-decoration {
  position: absolute;
  top: -10px;
  right: -10px;
  width: 24px;
  height: 24px;
  background: linear-gradient(135deg, #a9c3a6, #8fb58b);
  border-radius: 50%;
  opacity: 0.7;
  animation: float 3s ease-in-out infinite;
}

@keyframes float {
  0%, 100% { transform: translateY(0px) rotate(0deg); }
  50% { transform: translateY(-10px) rotate(180deg); }
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
  line-height: 1.6;
  max-width: 400px;
  margin: 0;
  opacity: 0.8;
}

/* 好友容器 */
.friends-container {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

/* 好友列表 */
.friends-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 13px;
  line-height: 1.4;
}

/* 好友条目 */
.friend-entry {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 16px;
  margin-bottom: 8px;
  background: #f8f6f0;
  border: 1px solid #f0ede6;
  border-radius: 15px;
  transition: all 0.3s ease;
  animation: friendEntryFadeIn 0.4s ease-out;
}

.friend-entry:hover {
  background: #f0ede6;
  border-color: #e4ddd3;
  transform: translateX(4px);
}

@keyframes friendEntryFadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 头像区域 */
.friend-avatar {
  position: relative;
  flex-shrink: 0;
}

.avatar-image {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid #e4ddd3;
  transition: all 0.3s ease;
}

.friend-entry:hover .avatar-image {
  border-color: #a9c3a6;
  transform: scale(1.05);
}

.avatar-status {
  position: absolute;
  bottom: 2px;
  right: 2px;
  width: 12px;
  height: 12px;
  background: #4caf50;
  border: 2px solid #fffcf6;
  border-radius: 50%;
}

/* 好友信息 */
.friend-info {
  flex: 1;
  min-width: 0;
}

.friend-name {
  font-size: 16px;
  font-weight: 600;
  color: #4a593d;
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.friend-details {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.friend-remark {
  font-size: 13px;
  color: #6e8b67;
  opacity: 0.8;
}

.friend-id {
  font-size: 12px;
  color: #a0a0a0;
  font-family: 'Monaco', 'Menlo', monospace;
}

/* 好友操作 */
.friend-actions {
  flex-shrink: 0;
}

.btn-message {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: #a9c3a6;
  color: white;
  border: none;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 2px 8px rgba(169, 195, 166, 0.3);
}

.btn-message:hover {
  background: #8fb58b;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(169, 195, 166, 0.4);
}

/* 好友底部 */
.friends-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  border-top: 1px solid #f0ede6;
  flex-shrink: 0;
  background: #fffcf6;
}

.footer-info {
  display: flex;
  gap: 16px;
  align-items: center;
}

.info-text {
  font-size: 13px;
  color: #6e8b67;
  opacity: 0.8;
}

.footer-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-page {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: #a9c3a6;
  color: white;
  border: none;
  border-radius: 50%;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 2px 6px rgba(169, 195, 166, 0.3);
}

.btn-page:hover:not(:disabled) {
  background: #8fb58b;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(169, 195, 166, 0.4);
}

.btn-page:disabled {
  background: #e4ddd3;
  color: #a0a0a0;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.page-info {
  font-size: 13px;
  color: #4a593d;
  font-weight: 500;
  min-width: 60px;
  text-align: center;
}

/* 自定义滚动条样式 */
.friends-list::-webkit-scrollbar {
  width: 8px;
}

.friends-list::-webkit-scrollbar-track {
  background: #f5f5f1;
  border-radius: 4px;
}

.friends-list::-webkit-scrollbar-thumb {
  background: #a9c3a6;
  border-radius: 4px;
  transition: background 0.3s ease;
}

.friends-list::-webkit-scrollbar-thumb:hover {
  background: #8fb58b;
}

/* 加载遮罩 */
.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(245, 245, 241, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

.loading-content {
  text-align: center;
  padding: 32px;
  background: #fffcf6;
  border-radius: 15px;
  border: 1px solid #e4ddd3;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid #e4ddd3;
  border-top: 3px solid #a9c3a6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 16px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.loading-text {
  font-size: 16px;
  color: #4a593d;
  font-weight: 500;
}

/* 错误提示 */
.error-toast {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 2000;
  animation: slideInRight 0.3s ease-out;
}

@keyframes slideInRight {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.error-content {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  background: #fff5f5;
  border: 1px solid #fed7d7;
  border-radius: 15px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  max-width: 400px;
}

.error-icon {
  font-size: 20px;
  color: #e53e3e;
}

.error-message {
  flex: 1;
  font-size: 14px;
  color: #742a2a;
  line-height: 1.4;
}

.error-close {
  background: none;
  border: none;
  font-size: 16px;
  color: #a0a0a0;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.error-close:hover {
  background: #fed7d7;
  color: #742a2a;
}
</style>