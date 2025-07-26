<script setup>
import { ref, computed } from "vue";
import { useRouter, useRoute } from 'vue-router';

// 侧边栏导航项
const navItems = [
  { id: 'monitor', name: '监控', icon: '📊', path: '/monitor' },
  { id: 'servers', name: '服务器列表', icon: '🖥️', path: '/servers' },
  { id: 'friends', name: '好友列表', icon: '👥', path: '/friends' },
  { id: 'groups', name: '群聊列表', icon: '💬', path: '/groups' },
  { id: 'logs', name: '日志', icon: '📝', path: '/logs' }
];

const router = useRouter();
const route = useRoute();

// 根据当前路由计算活跃导航项
const activeNav = computed(() => {
  const currentPath = route.path;
  const activeItem = navItems.find(item => item.path === currentPath);
  return activeItem ? activeItem.id : 'monitor';
});

const selectNav = (item) => {
  router.push(item.path);
};
</script>

<template>
  <div class="app-container">
    <!-- 左侧侧边栏 -->
    <aside class="sidebar">
      <div class="sidebar-header">
        <h1 class="app-title">LinBot2</h1>
      </div>
      
      <nav class="sidebar-nav">
        <ul class="nav-list">
          <li 
            v-for="item in navItems" 
            :key="item.id"
            class="nav-item"
            :class="{ active: activeNav === item.id }"
            @click="selectNav(item)"
          >
            <span class="nav-icon">{{ item.icon }}</span>
            <span class="nav-text">{{ item.name }}</span>
          </li>
        </ul>
      </nav>
    </aside>

    <!-- 右侧内容区 -->
    <main class="main-content">
      <div class="content-header">
        <h2 class="content-title">
          {{ navItems.find(item => item.id === activeNav)?.name || '首页' }}
        </h2>
      </div>
      
      <div class="content-body">
        <router-view />
      </div>
    </main>
  </div>
</template>

<style>
/* 全局样式 - 自然柔和风格 */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  font-family: "PingFang SC", "Microsoft YaHei", Arial, sans-serif;
  font-size: 14px;
  line-height: 1.6;
  font-weight: 400;
  
  /* 自然柔和色彩 */
  --bg-color: #f5f5f1;          /* 背景色淡米色 */
  --text-primary: #4a593d;      /* 主要文字深绿色 */
  --text-title: #6e8b67;        /* 标题中等绿色 */
  --card-bg: #fffcf6;           /* 卡片背景奶白色 */
  --border-color: #e4ddd3;      /* 边框淡棕色 */
  --button-bg: #a9c3a6;         /* 按钮淡绿色 */
  --button-hover: #8fb58b;      /* 按钮悬停较深绿色 */
  
  color: var(--text-primary);
  background-color: var(--bg-color);
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  margin: 0;
  min-height: 100vh;
}

.app-container {
  display: flex;
  height: 100vh;
  background-color: var(--bg-color);
}

/* 左侧侧边栏 */
.sidebar {
  width: 240px;
  background-color: var(--card-bg);
  border-right: 1px solid var(--border-color);
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.06);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  padding: 24px 20px;
  border-bottom: 1px solid var(--border-color);
  background: linear-gradient(135deg, var(--card-bg) 0%, #f8f6f0 100%);
}

.app-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-title);
  text-align: center;
  margin: 0;
}

.sidebar-nav {
  flex: 1;
  padding: 16px 0;
}

.nav-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.nav-item {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  margin: 4px 12px;
  border-radius: 15px;
  cursor: pointer;
  transition: all 0.3s ease;
  color: var(--text-primary);
}

.nav-item:hover {
  background-color: rgba(169, 195, 166, 0.1);
  transform: translateX(4px);
}

.nav-item.active {
  background-color: var(--button-bg);
  color: white;
  box-shadow: 0 4px 12px rgba(169, 195, 166, 0.3);
}

.nav-icon {
  font-size: 16px;
  margin-right: 12px;
  width: 20px;
  text-align: center;
}

.nav-text {
  font-size: 14px;
  font-weight: 500;
}

/* 右侧内容区 */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  background-color: var(--bg-color);
}

.content-header {
  padding: 21px 32px;
  background-color: var(--card-bg);
  border-bottom: 1px solid var(--border-color);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.content-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-title);
  margin: 0;
}

.content-body {
  flex: 1;
  padding: 32px;
  overflow-y: auto;
}

.welcome-message {
  background-color: var(--card-bg);
  padding: 40px;
  border-radius: 15px;
  border: 1px solid var(--border-color);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
  text-align: center;
  max-width: 600px;
  margin: 0 auto;
}

.welcome-message p {
  font-size: 18px;
  color: var(--text-title);
  margin-bottom: 12px;
  font-weight: 500;
}

.welcome-message .subtitle {
  font-size: 14px;
  color: var(--text-primary);
  opacity: 0.8;
  margin-bottom: 0;
}

/* 滚动条样式 */
::-webkit-scrollbar {
  width: 6px;
}

::-webkit-scrollbar-track {
  background: var(--border-color);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb {
  background: var(--button-bg);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--button-hover);
}

/* 页面占位符样式 */
.page-placeholder {
  background-color: var(--card-bg);
  padding: 60px 40px;
  border-radius: 15px;
  border: 1px solid var(--border-color);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
  text-align: center;
  max-width: 600px;
  margin: 0 auto;
  font-size: 18px;
  color: var(--text-title);
  font-weight: 500;
}
</style>
