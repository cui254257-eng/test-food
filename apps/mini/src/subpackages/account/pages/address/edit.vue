<script setup lang="ts">
import { onLoad } from '@dcloudio/uni-app'
import { reactive, ref } from 'vue'
import { getAddress, saveAddress, type AddressInput } from '@/api/addresses'

const addressId = ref<number>()
const loading = ref(false)
const saving = ref(false)
const form = reactive<AddressInput>({
  recipient_name: '',
  phone: '',
  province: '',
  city: '',
  district: '',
  detail: '',
  is_default: false,
})

onLoad(async (query) => {
  const rawId = query?.id
  const id = Number(rawId)
  if (!rawId || !Number.isInteger(id) || id < 1) {
    uni.setNavigationBarTitle({ title: '新增收货地址' })
    return
  }

  uni.setNavigationBarTitle({ title: '编辑收货地址' })
  addressId.value = id
  loading.value = true
  try {
    Object.assign(form, await getAddress(id))
  } catch (error) {
    uni.showToast({
      title: error instanceof Error ? error.message : '收货地址加载失败',
      icon: 'none',
    })
  } finally {
    loading.value = false
  }
})

function toggleDefault() {
  form.is_default = !form.is_default
}

async function submit() {
  if (saving.value) return
  if (!form.recipient_name.trim()) {
    uni.showToast({ title: '请填写收件人姓名', icon: 'none' })
    return
  }
  if (!/^1\d{10}$/.test(form.phone.trim())) {
    uni.showToast({ title: '请填写有效的 11 位手机号码', icon: 'none' })
    return
  }
  if (!form.province.trim() || !form.city.trim() || !form.district.trim() || !form.detail.trim()) {
    uni.showToast({ title: '请完整填写所在地区和详细地址', icon: 'none' })
    return
  }

  saving.value = true
  uni.showLoading({ title: '保存中…', mask: true })
  try {
    await saveAddress({
      ...form,
      recipient_name: form.recipient_name.trim(),
      phone: form.phone.trim(),
      province: form.province.trim(),
      city: form.city.trim(),
      district: form.district.trim(),
      detail: form.detail.trim(),
    }, addressId.value)
    uni.hideLoading()
    uni.showToast({ title: '保存成功', icon: 'success' })
    uni.navigateBack()
  } catch (error) {
    uni.hideLoading()
    uni.showToast({
      title: error instanceof Error ? error.message : '保存失败，请稍后重试',
      icon: 'none',
    })
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <view class="edit-page min-h-screen px-4 pb-10 pt-4">
    <view v-if="loading" class="loading-note">正在加载地址…</view>

    <view class="form-card">
      <view class="form-row">
        <text class="field-label">收件人</text>
        <input v-model="form.recipient_name" class="field-input" placeholder="请输入姓名" maxlength="40" />
      </view>
      <view class="form-row">
        <text class="field-label">手机号</text>
        <input v-model="form.phone" class="field-input" type="number" placeholder="请输入 11 位手机号" maxlength="11" />
      </view>
      <view class="form-row region-row">
        <text class="field-label">所在地区</text>
        <view class="region-inputs">
          <input v-model="form.province" class="field-input" placeholder="省 / 直辖市" maxlength="50" />
          <input v-model="form.city" class="field-input" placeholder="市" maxlength="50" />
          <input v-model="form.district" class="field-input" placeholder="区 / 县" maxlength="50" />
        </view>
      </view>
      <view class="form-row detail-row">
        <text class="field-label">详细地址</text>
        <textarea
          v-model="form.detail"
          class="detail-input"
          placeholder="街道、楼栋、门牌号等"
          maxlength="255"
          auto-height
        />
      </view>
    </view>

    <view class="default-row" @tap="toggleDefault">
      <view class="default-copy">
        <text class="default-title">设为默认地址</text>
        <text class="default-caption">下次下单时优先使用</text>
      </view>
      <view class="default-checkbox" :class="{ checked: form.is_default }">
        <text v-if="form.is_default">✓</text>
      </view>
    </view>

    <view class="save-button" :class="{ disabled: saving || loading }" @tap="submit">
      {{ saving ? '保存中…' : '保存地址' }}
    </view>
    <text class="form-footnote">收货地址仅对当前微信账号可见</text>
  </view>
</template>

<style scoped>
.edit-page { box-sizing: border-box; background: #f4f2ed; color: #302c26; }
.loading-note { margin: 0 0 10px; color: #969188; font-size: 11px; text-align: center; }
.form-card { overflow: hidden; padding: 0 16px; border-radius: 16px; background: #fff; box-shadow: 0 5px 18px rgb(49 39 22 / 4%); }
.form-row { display: flex; align-items: center; min-height: 54px; gap: 13px; border-bottom: 1px solid #f1efeb; }
.form-row:last-child { border-bottom: 0; }
.field-label { width: 62px; flex: 0 0 auto; color: #4d4941; font-size: 12px; }
.field-input { min-width: 0; flex: 1; color: #302c26; font-size: 12px; }
.field-input::placeholder, .detail-input::placeholder { color: #b2aea6; }
.region-row { align-items: flex-start; padding: 17px 0 14px; }
.region-row .field-label { padding-top: 4px; }
.region-inputs { display: flex; flex: 1; flex-direction: column; gap: 15px; }
.region-inputs .field-input { width: 100%; }
.detail-row { align-items: flex-start; padding: 17px 0; }
.detail-row .field-label { padding-top: 2px; }
.detail-input { box-sizing: border-box; min-height: 55px; flex: 1; color: #302c26; font-size: 12px; line-height: 1.6; }
.default-row { display: flex; align-items: center; justify-content: space-between; margin-top: 13px; padding: 15px 16px; border-radius: 15px; background: #fff; }
.default-copy { display: flex; flex-direction: column; gap: 5px; }
.default-title { color: #454139; font-size: 12px; font-weight: 600; }
.default-caption { color: #a09c94; font-size: 10px; }
.default-checkbox { display: grid; width: 20px; height: 20px; place-items: center; border: 1px solid #d9d5cc; border-radius: 50%; color: #fff; font-size: 12px; }
.default-checkbox.checked { border-color: #b86a49; background: #b86a49; }
.save-button { margin-top: 24px; padding: 13px; border-radius: 13px; background: #b86a49; color: #fff; font-size: 13px; font-weight: 600; text-align: center; box-shadow: 0 7px 16px rgb(184 106 73 / 18%); }
.save-button.disabled { opacity: .65; }
.form-footnote { display: block; margin-top: 13px; color: #aaa69d; font-size: 10px; text-align: center; }
</style>
