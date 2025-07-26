<template>
  <div class="page-placeholder">
    <div class="placeholder-content">
      <div class="placeholder-icon">📊</div>
      <h2 class="placeholder-title">监控页面</h2>
      <p class="placeholder-text">开发中...</p>
      <p class="placeholder-hint">这里将显示服务器运行状态和性能监控信息</p>
    </div>
  </div>
</template>

<script setup>
// 监控页面逻辑
</script>

<style scoped>
.page-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  background-color: var(--bg-color);
  color: var(--text-color);
}

.placeholder-content {
  text-align: center;
  max-width: 400px;
  padding: 40px;
}

.placeholder-icon {
  font-size: 64px;
  margin-bottom: 24px;
}

.placeholder-title {
  color: var(--title-color);
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 16px;
}

.placeholder-text {
  color: var(--text-color);
  font-size: 16px;
  margin-bottom: 12px;
}

.placeholder-hint {
  color: #888;
  font-size: 14px;
  line-height: 1.5;
}
</style> 