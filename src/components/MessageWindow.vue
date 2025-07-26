<template>
  <div class="message-window-overlay" @click="closeWindow">
    <div class="message-window" @click.stop>
      <!-- 窗口头部 -->
      <div class="message-header">
        <div class="contact-info">
          <img 
            :src="avatarUrl" 
            :alt="contactName"
            class="contact-avatar"
            @error="handleAvatarError"
          />
          <div class="contact-details">
            <div class="contact-name">{{ contactName }}</div>
            <div class="contact-id">{{ contactType === 'private' ? 'QQ' : '群号' }}: {{ contactId }}</div>
          </div>
        </div>
        <button @click="closeWindow" class="close-btn">✕</button>
      </div>



      <!-- 简化的输入区域 -->
      <div class="input-area">
        <div class="input-container">
          <input
            v-model="inputMessage"
            @keydown="handleKeyDown"
            placeholder="输入消息..."
            class="message-input"
            type="text"
          />
          <button
            @click="sendMessage()"
            :disabled="!inputMessage.trim() || sending"
            class="send-btn"
          >
            {{ sending ? '发送中...' : '发送' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Props
const props = defineProps({
  contactType: {
    type: String,
    required: true,
    validator: value => ['private', 'group'].includes(value)
  },
  contactId: {
    type: Number,
    required: true
  },
  contactName: {
    type: String,
    required: true
  },
  visible: {
    type: Boolean,
    default: false
  }
});

// Emits
const emit = defineEmits(['close', 'messageSent']);

// 响应式数据
const inputMessage = ref('');
const sending = ref(false);

// 计算属性
const avatarUrl = computed(() => {
  if (props.contactType === 'private') {
    return `https://q1.qlogo.cn/g?b=qq&nk=${props.contactId}&s=640`;
  } else {
    return `https://p.qlogo.cn/gh/${props.contactId}/${props.contactId}/640/`;
  }
});

// 方法
const closeWindow = () => {
  emit('close');
};

const handleAvatarError = (event) => {
  // 头像加载失败时使用默认头像
  event.target.src = 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDAiIGhlaWdodD0iNDAiIHZpZXdCb3g9IjAgMCA0MCA0MCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPGNpcmNsZSBjeD0iMjAiIGN5PSIyMCIgcj0iMjAiIGZpbGw9IiNFNUU3RUIiLz4KPGNpcmNsZSBjeD0iMjAiIGN5PSIxNiIgcj0iNiIgZmlsbD0iIzlDQTNBRiIvPgo8cGF0aCBkPSJNMzAgMzJDMzAgMjYuNDc3MSAyNS41MjI5IDIyIDIwIDIyQzE0LjQ3NzEgMjIgMTAgMjYuNDc3MSAxMCAzMiIgZmlsbD0iIzlDQTNBRiIvPgo8L3N2Zz4K';
};



const sendMessage = async () => {
  // 防护检查
  if (!inputMessage.value || typeof inputMessage.value !== 'string') {
    console.error('inputMessage.value 无效:', inputMessage.value);
    return;
  }
  if (!inputMessage.value.trim() || sending.value) {
    console.log('消息为空或正在发送中');
    return;
  }

  const messageText = inputMessage.value.trim();
  sending.value = true;

  try {
    // 调试信息
    console.log('发送消息参数:', {
      contactType: props.contactType,
      contactId: props.contactId,
      messageText: messageText,
      messageLength: messageText.length
    });

    let response;
    if (props.contactType === 'private') {
      response = await invoke('send_private_message', {
        userId: props.contactId,
        message: messageText
      });
    } else {
      response = await invoke('send_group_message', {
        groupId: props.contactId,
        message: messageText
      });
    }

    // 发送成功，清空输入框
    inputMessage.value = '';

    console.log('消息发送成功:', response);
    emit('messageSent', { messageText, response });
  } catch (error) {
    console.error('发送消息失败:', error);
    alert('发送消息失败: ' + error);
  } finally {
    sending.value = false;
  }
};

const handleKeyDown = (event) => {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault();
    sendMessage();
  }
};








</script>

<style scoped>
.message-window-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.message-window {
  width: 500px;
  background-color: var(--card-bg);
  border-radius: 15px;
  border: 1px solid var(--border-color);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 窗口头部 */
.message-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
  background-color: var(--card-bg);
}

.contact-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.contact-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid var(--border-color);
}

.contact-details {
  display: flex;
  flex-direction: column;
}

.contact-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-title);
  margin-bottom: 2px;
}

.contact-id {
  font-size: 12px;
  color: #888;
}

.close-btn {
  width: 32px;
  height: 32px;
  border: none;
  background-color: transparent;
  color: #888;
  font-size: 16px;
  cursor: pointer;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background-color: var(--border-color);
  color: var(--text-primary);
}



/* 输入区域 */
.input-area {
  border-top: 1px solid var(--border-color);
  background-color: var(--card-bg);
  padding: 20px;
}

.input-container {
  display: flex;
  gap: 12px;
  align-items: center;
}

.message-input {
  flex: 1;
  border: 1px solid var(--border-color);
  border-radius: 15px;
  padding: 12px 16px;
  font-size: 14px;
  font-family: inherit;
  background-color: var(--bg-color);
  color: var(--text-primary);
  transition: all 0.2s ease;
  height: 44px;
}

.message-input:focus {
  outline: none;
  border-color: var(--button-bg);
  box-shadow: 0 0 0 2px rgba(169, 195, 166, 0.2);
}

.send-btn {
  padding: 12px 24px;
  background-color: var(--button-bg);
  color: white;
  border: none;
  border-radius: 30px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
  height: 44px;
}

.send-btn:hover:not(:disabled) {
  background-color: var(--button-hover);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(169, 195, 166, 0.3);
}

.send-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
