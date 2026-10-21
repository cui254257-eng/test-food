<script setup lang="ts">
import { ref } from 'vue'
import { loginCustomer } from '@/api/auth'

const loading = ref(false)

async function wechatLogin() {
  if (loading.value) return
  loading.value = true
  try {
    await loginCustomer()
    uni.showToast({ title: '登录成功', icon: 'success' })
    setTimeout(() => {
      if (getCurrentPages().length > 1) uni.navigateBack()
      else uni.switchTab({ url: '/pages/profile/index' })
    }, 300)
  } catch (error) {
    uni.showModal({
      title: '微信登录失败',
      content: error instanceof Error ? error.message : '请稍后重试',
      showCancel: false,
      confirmColor: '#b86a49',
    })
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <view class="login-page">
    <view class="glow glow-top" />
    <view class="glow glow-bottom" />

    <view class="login-content">
      <view class="brand-icon" aria-label="好味小馆">
        <view class="icon-halo" />
        <view class="icon-plate"><view class="icon-center" /></view>
        <view class="icon-steam steam-one" />
        <view class="icon-steam steam-two" />
        <view class="icon-leaf">✦</view>
      </view>

      <text class="brand-name">好味小馆</text>
      <text class="page-title">欢迎登录</text>
      <text class="page-caption">选择登录方式</text>

      <view class="login-methods">
        <button class="method method-unavailable" disabled>
          <view class="method-icon one-tap-icon"><text>一</text></view>
          <text class="method-name">微信一键登录</text>
          <text class="method-status">暂不可用</text>
        </button>

        <button class="method method-wechat" :class="{ 'method-loading': loading }" :disabled="loading" @tap="wechatLogin">
          <view class="method-icon wechat-icon">
            <view class="wechat-bubble bubble-main"><view /><view /></view>
            <view class="wechat-bubble bubble-sub"><view /><view /></view>
          </view>
          <text class="method-name">{{ loading ? '正在登录…' : '微信登录' }}</text>
          <wd-icon name="arrow-right" size="16px" color="#8b9c8a" />
        </button>

        <button class="method method-unavailable" disabled>
          <view class="method-icon phone-icon"><text>☎</text></view>
          <text class="method-name">手机号登录</text>
          <text class="method-status">暂未开放</text>
        </button>
      </view>

      <text class="login-note">微信登录即自动创建或进入账号</text>
    </view>

    <view class="page-footer">
      <view class="footer-line" />
      <text>好味小馆 · 用心做好每一餐</text>
      <view class="footer-line" />
    </view>
  </view>
</template>

<style scoped>
.login-page { position: relative; display: flex; min-height: 100vh; box-sizing: border-box; overflow: hidden; align-items: center; justify-content: center; padding: 28px 24px calc(env(safe-area-inset-bottom) + 28px); background: #f7f3ed; color: #382b21; }
.glow { position: absolute; width: 260px; height: 260px; border-radius: 50%; pointer-events: none; }
.glow-top { top: -160px; right: -95px; background: radial-gradient(circle, rgb(226 179 143 / 23%), rgb(226 179 143 / 0%) 70%); }
.glow-bottom { bottom: -155px; left: -105px; background: radial-gradient(circle, rgb(198 211 181 / 24%), rgb(198 211 181 / 0%) 70%); }
.login-content { position: relative; z-index: 1; display: flex; width: 100%; max-width: 390px; align-items: center; flex-direction: column; margin-top: -18px; }
.brand-icon { position: relative; display: grid; width: 100px; height: 100px; place-items: center; margin-bottom: 17px; }
.icon-halo { position: absolute; inset: 0; border: 1px solid #ead7c3; border-radius: 34px; background: linear-gradient(145deg, #fffaf3, #f5e8d8); box-shadow: 0 12px 25px rgb(112 77 49 / 9%); transform: rotate(-7deg); }
.icon-plate { position: relative; display: grid; width: 53px; height: 53px; place-items: center; border: 3px solid #b86a49; border-radius: 50%; background: #fffaf4; box-shadow: inset 0 0 0 4px #f3e1d0; }
.icon-center { width: 22px; height: 22px; border: 2px solid #d89a73; border-radius: 50%; background: #f7e8d8; }
.icon-steam { position: absolute; top: 15px; width: 5px; height: 13px; border-left: 2px solid #b86a49; border-radius: 50%; transform: rotate(18deg); }
.steam-one { left: 39px; }
.steam-two { left: 54px; top: 12px; height: 16px; transform: rotate(-17deg); }
.icon-leaf { position: absolute; right: 17px; bottom: 18px; color: #7e906b; font-size: 13px; }
.brand-name { color: #a86a4b; font-size: 11px; font-weight: 700; letter-spacing: 3px; }
.page-title { margin-top: 12px; color: #392c22; font-family: serif; font-size: 25px; font-weight: 750; letter-spacing: 1px; }
.page-caption { margin-top: 7px; color: #9a9084; font-size: 11px; }
.login-methods { display: flex; width: 100%; flex-direction: column; gap: 12px; margin-top: 32px; }
.method { display: flex; box-sizing: border-box; width: 100%; height: 58px; align-items: center; gap: 12px; margin: 0; padding: 0 16px; border: 1px solid #eee7de; border-radius: 15px; background: #fffdfa; box-shadow: 0 5px 14px rgb(64 46 30 / 3%); color: inherit; line-height: normal; text-align: left; }
.method::after { border: 0; }
.method-icon { display: grid; width: 34px; height: 34px; flex: 0 0 34px; place-items: center; border-radius: 11px; }
.one-tap-icon { background: #f5f0e9; color: #a69a8b; font-size: 15px; font-weight: 700; }
.phone-icon { background: #f1f2eb; color: #899078; font-size: 18px; }
.method-name { flex: 1; color: #44382e; font-size: 13px; font-weight: 650; }
.method-status { color: #b4aca1; font-size: 10px; }
.method-unavailable { background: rgb(255 253 250 / 68%); box-shadow: none; }
.method-unavailable[disabled] { opacity: .68; }
.method-unavailable .method-name { color: #aaa196; font-weight: 500; }
.method-wechat { border-color: #cfe3d4; background: linear-gradient(100deg, #f8fcf8, #fffdfa); }
.method-wechat:active { background: #eef7f0; }
.method-loading { opacity: .7; }
.wechat-icon { position: relative; background: #e8f4e9; }
.wechat-bubble { position: absolute; display: flex; align-items: center; justify-content: center; gap: 3px; border-radius: 50%; background: #24a148; }
.wechat-bubble view { width: 2px; height: 2px; border-radius: 50%; background: #fff; }
.bubble-main { top: 8px; left: 6px; width: 19px; height: 15px; }
.bubble-sub { right: 4px; bottom: 7px; width: 15px; height: 12px; border: 1px solid #e8f4e9; background: #7bbb83; }
.login-note { margin-top: 21px; color: #aaa093; font-size: 10px; }
.page-footer { position: absolute; right: 0; bottom: calc(env(safe-area-inset-bottom) + 22px); left: 0; display: flex; align-items: center; justify-content: center; gap: 9px; color: #b5aa9b; font-size: 9px; letter-spacing: .5px; }
.footer-line { width: 20px; height: 1px; background: #d9cdbf; }
</style>
