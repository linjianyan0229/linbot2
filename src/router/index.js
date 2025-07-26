import { createRouter, createWebHashHistory } from 'vue-router'

// 路由配置
const routes = [
  {
    path: '/',
    redirect: '/monitor'
  },
  {
    path: '/monitor',
    name: 'Monitor',
    component: () => Promise.resolve({ template: '<div class="page-placeholder">监控页面 - 开发中</div>' })
  },
  {
    path: '/servers',
    name: 'Servers', 
    component: () => import('@/views/ServerList.vue')
  },
  {
    path: '/friends',
    name: 'Friends',
    component: () => Promise.resolve({ template: '<div class="page-placeholder">好友列表页面 - 开发中</div>' })
  },
  {
    path: '/groups',
    name: 'Groups',
    component: () => Promise.resolve({ template: '<div class="page-placeholder">群聊列表页面 - 开发中</div>' })
  },
  {
    path: '/logs',
    name: 'Logs',
    component: () => Promise.resolve({ template: '<div class="page-placeholder">日志页面 - 开发中</div>' })
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

export default router 