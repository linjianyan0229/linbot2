<template>
  <div class="startup-page">
    <!-- 主要内容区域 -->
    <div class="startup-content">
      <!-- 应用图标 -->
      <div class="app-icon">
        <img src="/icon.png" alt="LinBot2" class="icon-image" />
        <div class="icon-glow"></div>
      </div>

      <!-- 应用标题 -->
      <div class="app-title">
        <h1 class="title-main">LinBot2</h1>
        <p class="title-subtitle">LinBot 为你服务</p>
      </div>

      <!-- 进度条区域 -->
      <div class="progress-section">
        <div class="progress-container">
          <div class="progress-bar">
            <div 
              class="progress-fill" 
              :style="{ width: progress + '%' }"
            ></div>
            <div class="progress-shimmer"></div>
          </div>
          <div class="progress-text">{{ progressText }}</div>
        </div>
      </div>

      <!-- 状态信息 -->
      <div class="status-info">
        <div class="status-text">{{ statusText }}</div>
        <div v-if="currentVersion" class="version-info">
          当前版本: v{{ currentVersion }}
        </div>
      </div>
    </div>

    <!-- 版本更新对话框 -->
    <div v-if="showUpdateDialog" class="update-overlay">
      <div class="update-dialog">
        <div class="update-header">
          <div class="update-icon">🚀</div>
          <h3 class="update-title">发现新版本</h3>
        </div>
        
        <div class="update-content">
          <div class="version-comparison">
            <div class="version-item current">
              <span class="version-label">当前版本</span>
              <span class="version-number">v{{ currentVersion }}</span>
            </div>
            <div class="version-arrow">→</div>
            <div class="version-item latest">
              <span class="version-label">最新版本</span>
              <span class="version-number">v{{ latestVersion }}</span>
            </div>
          </div>
          
          <div v-if="releaseNotes" class="release-notes">
            <h4 class="notes-title">更新内容</h4>
            <div class="notes-content" v-html="releaseNotes"></div>
          </div>
        </div>
        
        <div class="update-actions">
          <button @click="downloadUpdate" class="btn-update">
            <span class="btn-icon">⬇️</span>
            <span class="btn-text">下载更新</span>
          </button>
          <button @click="skipUpdate" class="btn-skip">
            <span class="btn-text">跳过更新</span>
          </button>
        </div>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="errorMessage" class="error-toast">
      <div class="error-content">
        <span class="error-icon">⚠️</span>
        <span class="error-text">{{ errorMessage }}</span>
        <button @click="errorMessage = ''" class="error-close">✕</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';

const router = useRouter();

// 响应式数据
const progress = ref(0);
const progressText = ref('正在初始化...');
const statusText = ref('启动应用程序');
const currentVersion = ref('');
const latestVersion = ref('');
const showUpdateDialog = ref(false);
const releaseNotes = ref('');
const errorMessage = ref('');
const updateSkipped = ref(false); // 添加跳过更新的标记

// 启动流程控制
const startupSteps = [
  { text: '正在初始化...', duration: 1000 },
  { text: '正在加载配置...', duration: 800 },
  { text: '正在检查版本...', duration: 2000 },
  { text: '正在连接服务...', duration: 1200 },
  { text: '启动完成', duration: 500 }
];

let currentStep = 0;

// 版本比较函数
const compareVersions = (version1, version2) => {
  const v1Parts = version1.replace(/^v/, '').split('.').map(Number);
  const v2Parts = version2.replace(/^v/, '').split('.').map(Number);
  
  for (let i = 0; i < Math.max(v1Parts.length, v2Parts.length); i++) {
    const v1Part = v1Parts[i] || 0;
    const v2Part = v2Parts[i] || 0;
    
    if (v1Part < v2Part) return -1;
    if (v1Part > v2Part) return 1;
  }
  
  return 0;
};

// 获取应用版本
const getAppVersion = async () => {
  try {
    const version = await invoke('get_app_version');
    currentVersion.value = version;
    return version;
  } catch (error) {
    console.error('获取应用版本失败:', error);
    return null;
  }
};

// 检查最新版本
const checkLatestVersion = async () => {
  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 10000); // 10秒超时
    
    const response = await fetch(
      'https://api.github.com/repos/linjianyan0229/linbot2/releases/latest',
      { 
        signal: controller.signal,
        headers: {
          'Accept': 'application/vnd.github.v3+json'
        }
      }
    );
    
    clearTimeout(timeoutId);
    
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }
    
    const data = await response.json();
    return {
      version: data.tag_name,
      notes: data.body || '暂无更新说明'
    };
  } catch (error) {
    console.error('检查版本更新失败:', error);
    if (error.name === 'AbortError') {
      throw new Error('版本检查超时');
    }
    throw error;
  }
};

// 处理版本检查
const handleVersionCheck = async () => {
  // 如果用户已经跳过更新，直接继续
  if (updateSkipped.value) {
    return true;
  }

  try {
    statusText.value = '正在检查版本更新...';

    const [currentVer, latestInfo] = await Promise.all([
      getAppVersion(),
      checkLatestVersion()
    ]);

    if (currentVer && latestInfo) {
      latestVersion.value = latestInfo.version;

      const comparison = compareVersions(currentVer, latestInfo.version);

      if (comparison < 0) {
        // 有新版本可用
        releaseNotes.value = formatReleaseNotes(latestInfo.notes);
        showUpdateDialog.value = true;
        return false; // 暂停启动流程
      }
    }

    return true; // 继续启动流程
  } catch (error) {
    console.error('版本检查失败:', error);
    errorMessage.value = `版本检查失败: ${error.message}`;
    return true; // 继续启动流程
  }
};

// 格式化发布说明
const formatReleaseNotes = (notes) => {
  if (!notes) return '暂无更新说明';
  
  // 简单的 Markdown 转换
  return notes
    .replace(/^### (.*$)/gim, '<h5>$1</h5>')
    .replace(/^## (.*$)/gim, '<h4>$1</h4>')
    .replace(/^# (.*$)/gim, '<h3>$1</h3>')
    .replace(/^\* (.*$)/gim, '<li>$1</li>')
    .replace(/\n\n/g, '</p><p>')
    .replace(/^(.*)$/gim, '<p>$1</p>')
    .replace(/<p><li>/g, '<ul><li>')
    .replace(/<\/li><\/p>/g, '</li></ul>');
};

// 下载更新
const downloadUpdate = () => {
  const url = `https://github.com/linjianyan0229/linbot2/releases/latest`;
  window.open(url, '_blank');
  skipUpdate();
};

// 跳过更新
const skipUpdate = () => {
  updateSkipped.value = true; // 设置跳过标记
  showUpdateDialog.value = false;
  continueStartup();
};

// 继续启动流程
const continueStartup = async () => {
  for (let i = currentStep; i < startupSteps.length; i++) {
    const step = startupSteps[i];
    progressText.value = step.text;

    // 特殊处理版本检查步骤
    if (i === 2) {
      const shouldContinue = await handleVersionCheck();
      if (!shouldContinue && !updateSkipped.value) {
        currentStep = i;
        return; // 等待用户选择
      }
    }

    // 更新进度
    const targetProgress = ((i + 1) / startupSteps.length) * 100;
    await animateProgress(targetProgress, step.duration);

    currentStep = i + 1;
  }

  // 启动完成，跳转到监控页面
  statusText.value = '欢迎使用 LinBot2';
  await nextTick();
  setTimeout(() => {
    router.push('/monitor');
  }, 500);
};

// 进度条动画
const animateProgress = (targetProgress, duration) => {
  return new Promise((resolve) => {
    const startProgress = progress.value;
    const progressDiff = targetProgress - startProgress;
    const startTime = Date.now();
    
    const animate = () => {
      const elapsed = Date.now() - startTime;
      const progressRatio = Math.min(elapsed / duration, 1);
      
      // 使用缓动函数
      const easeOutQuart = 1 - Math.pow(1 - progressRatio, 4);
      progress.value = startProgress + (progressDiff * easeOutQuart);
      
      if (progressRatio < 1) {
        requestAnimationFrame(animate);
      } else {
        progress.value = targetProgress;
        resolve();
      }
    };
    
    requestAnimationFrame(animate);
  });
};

// 生命周期
onMounted(async () => {
  // 开始启动流程
  await continueStartup();
});
</script>

<style scoped>
/* 启动页面容器 */
.startup-page {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: linear-gradient(135deg, #f5f5f1 0%, #fffcf6 50%, #f8f6f0 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  z-index: 9999;
}

/* 主要内容区域 */
.startup-content {
  text-align: center;
  max-width: 500px;
  padding: 40px;
  animation: fadeInUp 1s ease-out;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 应用图标 */
.app-icon {
  position: relative;
  margin-bottom: 32px;
  display: inline-block;
}

.icon-image {
  width: 120px;
  height: 120px;
  border-radius: 30px;
  box-shadow: 0 8px 32px rgba(169, 195, 166, 0.3);
  animation: iconFloat 3s ease-in-out infinite;
  transition: all 0.3s ease;
}

.icon-glow {
  position: absolute;
  top: -10px;
  left: -10px;
  right: -10px;
  bottom: -10px;
  background: radial-gradient(circle, rgba(169, 195, 166, 0.2) 0%, transparent 70%);
  border-radius: 40px;
  animation: glowPulse 2s ease-in-out infinite alternate;
}

@keyframes iconFloat {
  0%, 100% { transform: translateY(0px); }
  50% { transform: translateY(-10px); }
}

@keyframes glowPulse {
  0% { opacity: 0.5; transform: scale(1); }
  100% { opacity: 0.8; transform: scale(1.1); }
}

/* 应用标题 */
.app-title {
  margin-bottom: 40px;
}

.title-main {
  font-size: 48px;
  font-weight: 700;
  color: #4a593d;
  margin: 0 0 8px 0;
  letter-spacing: -1px;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.title-subtitle {
  font-size: 18px;
  color: #6e8b67;
  margin: 0;
  opacity: 0.8;
  font-weight: 500;
}

/* 进度条区域 */
.progress-section {
  margin-bottom: 32px;
}

.progress-container {
  max-width: 400px;
  margin: 0 auto;
}

.progress-bar {
  position: relative;
  width: 100%;
  height: 8px;
  background: #e4ddd3;
  border-radius: 4px;
  overflow: hidden;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.1);
  margin-bottom: 16px;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #a9c3a6, #8fb58b);
  border-radius: 4px;
  transition: width 0.3s ease;
  position: relative;
}

.progress-shimmer {
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.4), transparent);
  animation: shimmer 2s infinite;
}

@keyframes shimmer {
  0% { left: -100%; }
  100% { left: 100%; }
}

.progress-text {
  font-size: 14px;
  color: #4a593d;
  font-weight: 500;
  margin-bottom: 8px;
}

/* 状态信息 */
.status-info {
  margin-bottom: 20px;
}

.status-text {
  font-size: 16px;
  color: #6e8b67;
  margin-bottom: 8px;
  font-weight: 500;
}

.version-info {
  font-size: 12px;
  color: #a0a0a0;
  font-family: 'Monaco', 'Menlo', monospace;
}

/* 更新对话框遮罩 */
.update-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(245, 245, 241, 0.95);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  backdrop-filter: blur(8px);
  animation: overlayFadeIn 0.3s ease-out;
}

@keyframes overlayFadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

/* 更新对话框 */
.update-dialog {
  background: #fffcf6;
  border-radius: 20px;
  border: 1px solid #e4ddd3;
  box-shadow: 0 16px 64px rgba(0, 0, 0, 0.15);
  max-width: 500px;
  width: 90%;
  max-height: 80vh;
  overflow: hidden;
  animation: dialogSlideIn 0.4s ease-out;
}

@keyframes dialogSlideIn {
  from {
    opacity: 0;
    transform: translateY(-20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

/* 更新对话框头部 */
.update-header {
  padding: 24px 24px 16px;
  text-align: center;
  border-bottom: 1px solid #f0ede6;
}

.update-icon {
  font-size: 48px;
  margin-bottom: 12px;
  animation: iconBounce 0.6s ease-out;
}

@keyframes iconBounce {
  0%, 20%, 50%, 80%, 100% { transform: translateY(0); }
  40% { transform: translateY(-10px); }
  60% { transform: translateY(-5px); }
}

.update-title {
  font-size: 24px;
  font-weight: 600;
  color: #4a593d;
  margin: 0;
}

/* 更新对话框内容 */
.update-content {
  padding: 20px 24px;
}

.version-comparison {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  margin-bottom: 24px;
  padding: 16px;
  background: #f8f6f0;
  border-radius: 15px;
  border: 1px solid #f0ede6;
}

.version-item {
  text-align: center;
  flex: 1;
}

.version-label {
  display: block;
  font-size: 12px;
  color: #6e8b67;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 4px;
}

.version-number {
  display: block;
  font-size: 18px;
  font-weight: 700;
  font-family: 'Monaco', 'Menlo', monospace;
}

.version-item.current .version-number {
  color: #a0a0a0;
}

.version-item.latest .version-number {
  color: #4a593d;
}

.version-arrow {
  font-size: 20px;
  color: #a9c3a6;
  font-weight: bold;
  animation: arrowPulse 1.5s ease-in-out infinite;
}

@keyframes arrowPulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.2); }
}

/* 发布说明 */
.release-notes {
  margin-top: 20px;
}

.notes-title {
  font-size: 16px;
  font-weight: 600;
  color: #4a593d;
  margin: 0 0 12px 0;
}

.notes-content {
  max-height: 200px;
  overflow-y: auto;
  padding: 16px;
  background: #f8f6f0;
  border-radius: 15px;
  border: 1px solid #f0ede6;
  font-size: 14px;
  line-height: 1.6;
  color: #4a593d;
}

.notes-content::-webkit-scrollbar {
  width: 6px;
}

.notes-content::-webkit-scrollbar-track {
  background: #e4ddd3;
  border-radius: 3px;
}

.notes-content::-webkit-scrollbar-thumb {
  background: #a9c3a6;
  border-radius: 3px;
}

/* 更新对话框操作按钮 */
.update-actions {
  padding: 16px 24px 24px;
  display: flex;
  gap: 12px;
  justify-content: center;
}

.btn-update,
.btn-skip {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  border: none;
  border-radius: 30px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  min-width: 120px;
  justify-content: center;
}

.btn-update {
  background: #a9c3a6;
  color: white;
  box-shadow: 0 4px 16px rgba(169, 195, 166, 0.3);
}

.btn-update:hover {
  background: #8fb58b;
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(169, 195, 166, 0.4);
}

.btn-skip {
  background: #f0ede6;
  color: #6e8b67;
  border: 1px solid #e4ddd3;
}

.btn-skip:hover {
  background: #e4ddd3;
  transform: translateY(-1px);
}

/* 错误提示 */
.error-toast {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 11000;
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
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  max-width: 400px;
  min-width: 300px;
}

.error-icon {
  font-size: 20px;
  color: #e53e3e;
  flex-shrink: 0;
}

.error-text {
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
  flex-shrink: 0;
}

.error-close:hover {
  background: #fed7d7;
  color: #742a2a;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .startup-content {
    padding: 20px;
    max-width: 90%;
  }

  .title-main {
    font-size: 36px;
  }

  .title-subtitle {
    font-size: 16px;
  }

  .icon-image {
    width: 100px;
    height: 100px;
  }

  .update-dialog {
    margin: 20px;
    width: calc(100% - 40px);
  }

  .update-actions {
    flex-direction: column;
  }

  .btn-update,
  .btn-skip {
    width: 100%;
  }

  .error-toast {
    top: 10px;
    right: 10px;
    left: 10px;
  }

  .error-content {
    min-width: auto;
    max-width: none;
  }
}

/* 深色模式支持 */
@media (prefers-color-scheme: dark) {
  .startup-page {
    background: linear-gradient(135deg, #2d3748 0%, #4a5568 50%, #2d3748 100%);
  }

  .title-main {
    color: #e2e8f0;
  }

  .title-subtitle,
  .status-text {
    color: #a0aec0;
  }

  .progress-text {
    color: #e2e8f0;
  }
}
</style>
