"use strict";
const common_vendor = require("../../../../common/vendor.js");
const api_addresses = require("../../../../api/addresses.js");
const _sfc_main = /* @__PURE__ */ common_vendor.defineComponent({
  __name: "edit",
  setup(__props) {
    const addressId = common_vendor.ref();
    const loading = common_vendor.ref(false);
    const saving = common_vendor.ref(false);
    const form = common_vendor.reactive({
      recipient_name: "",
      phone: "",
      province: "",
      city: "",
      district: "",
      detail: "",
      is_default: false
    });
    common_vendor.onLoad(async (query) => {
      const rawId = query == null ? void 0 : query.id;
      const id = Number(rawId);
      if (!rawId || !Number.isInteger(id) || id < 1) {
        common_vendor.index.setNavigationBarTitle({ title: "新增收货地址" });
        return;
      }
      common_vendor.index.setNavigationBarTitle({ title: "编辑收货地址" });
      addressId.value = id;
      loading.value = true;
      try {
        Object.assign(form, await api_addresses.getAddress(id));
      } catch (error) {
        common_vendor.index.showToast({
          title: error instanceof Error ? error.message : "收货地址加载失败",
          icon: "none"
        });
      } finally {
        loading.value = false;
      }
    });
    function toggleDefault() {
      form.is_default = !form.is_default;
    }
    async function submit() {
      if (saving.value)
        return;
      if (!form.recipient_name.trim()) {
        common_vendor.index.showToast({ title: "请填写收件人姓名", icon: "none" });
        return;
      }
      if (!/^1\d{10}$/.test(form.phone.trim())) {
        common_vendor.index.showToast({ title: "请填写有效的 11 位手机号码", icon: "none" });
        return;
      }
      if (!form.province.trim() || !form.city.trim() || !form.district.trim() || !form.detail.trim()) {
        common_vendor.index.showToast({ title: "请完整填写所在地区和详细地址", icon: "none" });
        return;
      }
      saving.value = true;
      common_vendor.index.showLoading({ title: "保存中…", mask: true });
      try {
        await api_addresses.saveAddress({
          ...form,
          recipient_name: form.recipient_name.trim(),
          phone: form.phone.trim(),
          province: form.province.trim(),
          city: form.city.trim(),
          district: form.district.trim(),
          detail: form.detail.trim()
        }, addressId.value);
        common_vendor.index.hideLoading();
        common_vendor.index.showToast({ title: "保存成功", icon: "success" });
        common_vendor.index.navigateBack();
      } catch (error) {
        common_vendor.index.hideLoading();
        common_vendor.index.showToast({
          title: error instanceof Error ? error.message : "保存失败，请稍后重试",
          icon: "none"
        });
      } finally {
        saving.value = false;
      }
    }
    return (_ctx, _cache) => {
      return common_vendor.e({
        a: loading.value
      }, loading.value ? {} : {}, {
        b: form.recipient_name,
        c: common_vendor.o(($event) => form.recipient_name = $event.detail.value, "e9"),
        d: form.phone,
        e: common_vendor.o(($event) => form.phone = $event.detail.value, "a0"),
        f: form.province,
        g: common_vendor.o(($event) => form.province = $event.detail.value, "e6"),
        h: form.city,
        i: common_vendor.o(($event) => form.city = $event.detail.value, "6d"),
        j: form.district,
        k: common_vendor.o(($event) => form.district = $event.detail.value, "89"),
        l: form.detail,
        m: common_vendor.o(($event) => form.detail = $event.detail.value, "b7"),
        n: form.is_default
      }, form.is_default ? {} : {}, {
        o: form.is_default ? 1 : "",
        p: common_vendor.o(toggleDefault, "2e"),
        q: common_vendor.t(saving.value ? "保存中…" : "保存地址"),
        r: saving.value || loading.value ? 1 : "",
        s: common_vendor.o(submit, "89")
      });
    };
  }
});
const MiniProgramPage = /* @__PURE__ */ common_vendor._export_sfc(_sfc_main, [["__scopeId", "data-v-e3be7d7d"]]);
wx.createPage(MiniProgramPage);
