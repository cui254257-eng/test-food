<script setup lang="ts">
import { onLoad } from '@dcloudio/uni-app'
import { ref } from 'vue'
import { getCustomerProfile, updateCustomerNickname, uploadCustomerAvatar } from '@/api/profile'

const nickname = ref('')
const savedNickname = ref('')
const avatarDataUrl = ref('')
const avatarPreview = ref('')
const pendingAvatarPath = ref('')
const loading = ref(true)
const compressingAvatar = ref(false)
const saving = ref(false)

onLoad(() => {
  void loadProfile()
})

async function loadProfile() {
  loading.value = true
  try {
    const profile = await getCustomerProfile()
    nickname.value = profile.nickname
    savedNickname.value = profile.nickname
    avatarDataUrl.value = profile.avatar_data_url ?? ''
    avatarPreview.value = avatarDataUrl.value
  } catch (error) {
    uni.showToast({ title: error instanceof Error ? error.message : '资料加载失败', icon: 'none' })
  } finally {
    loading.value = false
  }
}

function onChooseAvatar(event: { detail?: { avatarUrl?: string } }) {
  const source = event.detail?.avatarUrl
  if (!source) return

  compressingAvatar.value = true
  avatarPreview.value = source
  uni.compressImage({
    src: source,
    quality: 78,
    compressedWidth: 320,
    compressedHeight: 320,
    success: (result) => {
      pendingAvatarPath.value = result.tempFilePath
      avatarPreview.value = result.tempFilePath
      compressingAvatar.value = false
    },
    fail: () => {
      // Keep the original chooser path; the API still validates the actual image and size.
      pendingAvatarPath.value = source
      compressingAvatar.value = false
    },
  })
}

async function saveProfile() {
  const value = nickname.value.trim()
  if (!value) {
    uni.showToast({ title: '请输入昵称', icon: 'none' })
    return
  }
  if (value.length > 32) {
    uni.showToast({ title: '昵称不能超过 32 个字符', icon: 'none' })
    return
  }

  if (compressingAvatar.value) {
    uni.showToast({ title: '头像处理中，请稍候', icon: 'none' })
    return
  }

  saving.value = true
  try {
    if (pendingAvatarPath.value) {
      const updated = await uploadCustomerAvatar(pendingAvatarPath.value)
      avatarDataUrl.value = updated.avatar_data_url ?? ''
      pendingAvatarPath.value = ''
    }
    const updated = await updateCustomerNickname(value)
    savedNickname.value = updated.nickname
    nickname.value = updated.nickname
    avatarDataUrl.value = updated.avatar_data_url ?? avatarDataUrl.value
    uni.showToast({ title: '资料已保存', icon: 'success' })
    setTimeout(() => uni.navigateBack(), 350)
  } catch (error) {
    uni.showModal({
      title: '保存失败',
      content: error instanceof Error ? error.message : '请稍后重试',
      showCancel: false,
      confirmColor: '#b86a49',
    })
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <view class="edit-page">
    <view class="page-heading">
      <text class="page-kicker">PERSONAL PROFILE</text>
      <text class="page-title">编辑资料</text>
      <text class="page-caption">头像和昵称会保存到你的顾客账号</text>
    </view>

    <view class="profile-form">
      <view class="avatar-section">
        <button class="avatar-button" open-type="chooseAvatar" @chooseavatar="onChooseAvatar">
          <image v-if="avatarPreview" class="avatar-image" :src="avatarPreview" mode="aspectFill" />
          <view v-else class="avatar-placeholder">{{ nickname.slice(0, 1) || '微' }}</view>
          <view class="avatar-edit-mark">更换头像</view>
        </button>
        <text class="avatar-caption">点击头像选择图片</text>
      </view>

      <view class="form-divider" />

      <view class="nickname-section">
        <text class="field-label">昵称</text>
        <input
          v-model="nickname"
          class="nickname-input"
          type="nickname"
          maxlength="32"
          placeholder="微信用户xxx"
          placeholder-class="input-placeholder"
          :disabled="loading || saving"
        />
        <text class="field-hint">最多 32 个字符</text>
      </view>
    </view>

    <button class="save-button" :disabled="loading || saving || compressingAvatar" @tap="saveProfile">
      {{ saving ? '正在保存…' : '保存资料' }}
    </button>
    <text v-if="savedNickname" class="saved-note">当前昵称：{{ savedNickname }}</text>
  </view>
</template>

<style scoped>
.edit-page { min-height: 100vh; box-sizing: border-box; padding: calc(env(safe-area-inset-top) + 22px) 18px 36px; background: #f5f2ec; color: #34291f; }
.page-heading { display: flex; flex-direction: column; gap: 6px; margin: 2px 3px 22px; }
.page-kicker { color: #ae8b70; font-size: 9px; font-weight: 700; letter-spacing: 2px; }
.page-title { color: #33271e; font-family: serif; font-size: 26px; font-weight: 700; }
.page-caption { color: #9a8d7e; font-size: 11px; }
.profile-form { overflow: hidden; border: 1px solid #eee8df; border-radius: 20px; background: #fffdfa; box-shadow: 0 7px 22px rgb(54 41 29 / 4%); }
.avatar-section { display: flex; align-items: center; flex-direction: column; padding: 25px 16px 22px; }
.avatar-button { position: relative; display: flex; width: 94px; height: 94px; align-items: center; justify-content: center; overflow: visible; margin: 0; padding: 0; border: 0; border-radius: 50%; background: #f5e8d8; line-height: normal; }
.avatar-button::after { border: 0; }
.avatar-image, .avatar-placeholder { width: 90px; height: 90px; border: 2px solid #fff8ef; border-radius: 50%; background: #fff5e8; box-shadow: 0 4px 12px rgb(86 55 33 / 11%); }
.avatar-placeholder { display: flex; align-items: center; justify-content: center; color: #aa6245; font-family: serif; font-size: 36px; font-weight: 700; }
.avatar-edit-mark { position: absolute; right: -29px; bottom: 1px; padding: 5px 8px; border-radius: 10px; background: #fffaf3; color: #9a5638; font-size: 9px; box-shadow: 0 3px 9px rgb(104 64 39 / 9%); }
.avatar-caption { margin-top: 11px; color: #aaa093; font-size: 10px; }
.form-divider { height: 1px; margin: 0 17px; background: #f1ece5; }
.nickname-section { display: flex; flex-direction: column; padding: 18px 18px 19px; }
.field-label { margin-bottom: 10px; color: #514337; font-size: 12px; font-weight: 700; }
.nickname-input { height: 43px; box-sizing: border-box; padding: 0 12px; border: 1px solid #eee7de; border-radius: 11px; background: #fff; color: #382b21; font-size: 13px; }
.input-placeholder { color: #b8afa4; }
.field-hint { margin-top: 7px; color: #aaa093; font-size: 9px; text-align: right; }
.save-button { display: flex; height: 46px; align-items: center; justify-content: center; margin: 22px 0 0; border: 0; border-radius: 14px; background: #b86a49; box-shadow: 0 7px 16px rgb(143 78 50 / 16%); color: #fffaf4; font-size: 13px; font-weight: 700; line-height: 46px; }
.save-button::after { border: 0; }
.save-button[disabled] { opacity: .6; }
.saved-note { display: block; margin-top: 12px; color: #aaa093; font-size: 10px; text-align: center; }
</style>
