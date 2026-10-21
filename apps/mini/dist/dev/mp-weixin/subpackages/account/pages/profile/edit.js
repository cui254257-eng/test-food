"use strict";
const common_vendor = require("../../../../common/vendor.js");
const api_profile = require("../../../../api/profile.js");
const _sfc_main = /* @__PURE__ */ common_vendor.defineComponent({
  __name: "edit",
  setup(__props) {
    const nickname = common_vendor.ref("");
    const savedNickname = common_vendor.ref("");
    const avatarDataUrl = common_vendor.ref("");
    const avatarPreview = common_vendor.ref("");
    const pendingAvatarPath = common_vendor.ref("");
    const loading = common_vendor.ref(true);
    const compressingAvatar = common_vendor.ref(false);
    const saving = common_vendor.ref(false);
    common_vendor.onLoad(() => {
      void loadProfile();
    });
    async function loadProfile() {
      loading.value = true;
      try {
        const profile = await api_profile.getCustomerProfile();
        nickname.value = profile.nickname;
        savedNickname.value = profile.nickname;
        avatarDataUrl.value = profile.avatar_data_url ?? "";
        avatarPreview.value = avatarDataUrl.value;
      } catch (error) {
        common_vendor.index.showToast({ title: error instanceof Error ? error.message : "资料加载失败", icon: "none" });
      } finally {
        loading.value = false;
      }
    }
    function onChooseAvatar(event) {
      var _a;
      const source = (_a = event.detail) == null ? void 0 : _a.avatarUrl;
      if (!source)
        return;
      compressingAvatar.value = true;
      avatarPreview.value = source;
      common_vendor.index.compressImage({
        src: source,
        quality: 78,
        compressedWidth: 320,
        compressedHeight: 320,
        success: (result) => {
          pendingAvatarPath.value = result.tempFilePath;
          avatarPreview.value = result.tempFilePath;
          compressingAvatar.value = false;
        },
        fail: () => {
          pendingAvatarPath.value = source;
          compressingAvatar.value = false;
        }
      });
    }
    async function saveProfile() {
      const value = nickname.value.trim();
      if (!value) {
        common_vendor.index.showToast({ title: "请输入昵称", icon: "none" });
        return;
      }
      if (value.length > 32) {
        common_vendor.index.showToast({ title: "昵称不能超过 32 个字符", icon: "none" });
        return;
      }
      if (compressingAvatar.value) {
        common_vendor.index.showToast({ title: "头像处理中，请稍候", icon: "none" });
        return;
      }
      saving.value = true;
      try {
        if (pendingAvatarPath.value) {
          const updated2 = await api_profile.uploadCustomerAvatar(pendingAvatarPath.value);
          avatarDataUrl.value = updated2.avatar_data_url ?? "";
          pendingAvatarPath.value = "";
        }
        const updated = await api_profile.updateCustomerNickname(value);
        savedNickname.value = updated.nickname;
        nickname.value = updated.nickname;
        avatarDataUrl.value = updated.avatar_data_url ?? avatarDataUrl.value;
        common_vendor.index.showToast({ title: "资料已保存", icon: "success" });
        setTimeout(() => common_vendor.index.navigateBack(), 350);
      } catch (error) {
        common_vendor.index.showModal({
          title: "保存失败",
          content: error instanceof Error ? error.message : "请稍后重试",
          showCancel: false,
          confirmColor: "#b86a49"
        });
      } finally {
        saving.value = false;
      }
    }
    return (_ctx, _cache) => {
      return common_vendor.e({
        a: avatarPreview.value
      }, avatarPreview.value ? {
        b: avatarPreview.value
      } : {
        c: common_vendor.t(nickname.value.slice(0, 1) || "微")
      }, {
        d: common_vendor.o(onChooseAvatar, "96"),
        e: loading.value || saving.value,
        f: nickname.value,
        g: common_vendor.o(($event) => nickname.value = $event.detail.value, "46"),
        h: common_vendor.t(saving.value ? "正在保存…" : "保存资料"),
        i: loading.value || saving.value || compressingAvatar.value,
        j: common_vendor.o(saveProfile, "29"),
        k: savedNickname.value
      }, savedNickname.value ? {
        l: common_vendor.t(savedNickname.value)
      } : {});
    };
  }
});
const MiniProgramPage = /* @__PURE__ */ common_vendor._export_sfc(_sfc_main, [["__scopeId", "data-v-9d216046"]]);
wx.createPage(MiniProgramPage);
