import { createRouter, createWebHashHistory } from 'vue-router'

// 路由配置
const routes = [
  {
    path: '/',
    redirect: '/startup'
  },
  {
    path: '/startup',
    name: 'Startup',
    component: () => import('@/views/Startup.vue')
  },
  {
    path: '/monitor',
    name: 'Monitor',
    component: () => import('@/views/Monitor.vue')
  },
  {
    path: '/servers',
    name: 'Servers',
    component: () => import('@/views/ServerList.vue')
  },
  {
    path: '/friends',
    name: 'Friends',
    component: () => import('@/views/Friends.vue')
  },
  {
    path: '/groups',
    name: 'Groups',
    component: () => import('@/views/Groups.vue')
  },
  {
    path: '/logs',
    name: 'Logs',
    component: () => import('@/views/Logs.vue')
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

export default router 