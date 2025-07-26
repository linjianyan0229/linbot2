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
        <div class="logo-container">
          <img src="./assets/icon.png" alt="LinBot2 Logo" class="app-logo"/>
        </div>
        <h1 class="app-title">LinBot2</h1>
        <p class="app-subtitle">智能机器人管理平台</p>
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
      <router-view />
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
  position: relative;
  overflow: hidden;
}

.sidebar::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent);
  transition: left 0.5s ease;
}

.sidebar:hover::before {
  left: 100%;
}

.sidebar-header {
  padding: 32px 20px 24px;
  border-bottom: 1px solid var(--border-color);
  background: linear-gradient(135deg, var(--card-bg) 0%, #f8f6f0 100%);
  text-align: center;
  position: relative;
}

.sidebar-header::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 4px;
  background: linear-gradient(90deg, var(--button-bg) 0%, var(--button-hover) 100%);
}

.logo-container {
  display: flex;
  justify-content: center;
  align-items: center;
  margin-bottom: 16px;
  position: relative;
}

.app-logo {
  width: 64px;
  height: 64px;
  border-radius: 15px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  transition: all 0.3s ease;
  background: white;
  padding: 8px;
}

.app-logo:hover {
  transform: translateY(-2px) scale(1.05);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.18);
}

.app-title {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-title);
  margin: 0 0 8px 0;
  letter-spacing: 0.5px;
  background: linear-gradient(135deg, var(--text-title) 0%, var(--button-bg) 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.app-subtitle {
  font-size: 12px;
  color: #888;
  margin: 0;
  font-weight: 400;
  opacity: 0.8;
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
  padding: 14px 20px;
  margin: 6px 12px;
  border-radius: 15px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  color: var(--text-primary);
  position: relative;
  overflow: hidden;
}

.nav-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  width: 4px;
  height: 100%;
  background: var(--button-bg);
  transform: scaleY(0);
  transition: transform 0.3s ease;
}

.nav-item:hover {
  background-color: rgba(169, 195, 166, 0.1);
  transform: translateX(6px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.nav-item:hover::before {
  transform: scaleY(1);
}

.nav-item.active {
  background: linear-gradient(135deg, var(--button-bg) 0%, var(--button-hover) 100%);
  color: white;
  box-shadow: 0 6px 20px rgba(169, 195, 166, 0.4);
  transform: translateX(6px);
}

.nav-item.active::before {
  transform: scaleY(1);
  background: white;
}

.nav-icon {
  font-size: 18px;
  margin-right: 14px;
  width: 24px;
  text-align: center;
  transition: all 0.3s ease;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.1));
}

.nav-item:hover .nav-icon {
  transform: scale(1.1);
}

.nav-item.active .nav-icon {
  transform: scale(1.15);
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.2));
}

.nav-text {
  font-size: 14px;
  font-weight: 500;
  transition: all 0.3s ease;
  letter-spacing: 0.3px;
}

.nav-item.active .nav-text {
  font-weight: 600;
}

/* 右侧内容区 */
.main-content {
  flex: 1;
  background-color: var(--bg-color);
  overflow-y: auto;
  height: 100vh;
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
