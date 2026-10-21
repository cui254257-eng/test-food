"use strict";
const common_vendor = require("../../../../../common/vendor.js");
if (!Math) {
  wdIcon();
}
const wdIcon = () => "../wd-icon/wd-icon.js";
const __default__ = {
  name: "wd-notice-bar",
  options: {
    virtualHost: true,
    addGlobalClass: true,
    styleIsolation: "shared"
  }
};
const _sfc_main = /* @__PURE__ */ common_vendor.defineComponent({
  ...__default__,
  props: common_vendor.noticeBarProps,
  emits: ["close", "next", "click"],
  setup(__props, { expose: __expose, emit: __emit }) {
    const $wrap = ".wd-notice-bar__wrap";
    const $content = ".wd-notice-bar__content";
    const props = __props;
    const emit = __emit;
    const wrapWidth = common_vendor.ref(0);
    const show = common_vendor.ref(true);
    const currentIndex = common_vendor.ref(0);
    const textArray = common_vendor.computed(() => Array.isArray(props.text) ? props.text : [props.text]);
    const currentText = common_vendor.computed(() => textArray.value[currentIndex.value]);
    const verticalIndex = common_vendor.ref(0);
    const wrapRect = common_vendor.ref(null);
    const contentRect = common_vendor.ref(null);
    const isHorizontal = common_vendor.computed(() => props.direction === "horizontal");
    const isVertical = common_vendor.computed(() => props.direction === "vertical");
    const transitionState = common_vendor.reactive({
      transitionProperty: "unset",
      transitionDelay: "unset",
      transitionDuration: "unset",
      transform: "none",
      transitionTimingFunction: "linear"
    });
    const animation = common_vendor.computed(() => {
      return common_vendor.objToStyle(transitionState);
    });
    const rootStyle = common_vendor.computed(() => {
      const style = {};
      if (common_vendor.isDef(props.color)) {
        style.color = props.color;
      }
      if (common_vendor.isDef(props.backgroundColor)) {
        style.background = props.backgroundColor;
      }
      return `${common_vendor.objToStyle(style)}${props.customStyle}`;
    });
    const noticeBarClass = common_vendor.computed(() => {
      const { type, wrapable, scrollable } = props;
      let noticeBarClasses = [];
      type && noticeBarClasses.push(`is-${type}`);
      if (isHorizontal.value) {
        !wrapable && !scrollable && noticeBarClasses.push("wd-notice-bar--ellipse");
      } else {
        noticeBarClasses.push("wd-notice-bar--ellipse");
      }
      wrapable && !scrollable && noticeBarClasses.push("wd-notice-bar--wrap");
      return noticeBarClasses.join(" ");
    });
    const { proxy } = common_vendor.getCurrentInstance();
    common_vendor.watch(
      () => props.text,
      () => {
        reset();
      },
      { deep: true }
    );
    common_vendor.onMounted(() => {
      startTransition();
    });
    common_vendor.onShow(() => {
      startTransition();
    });
    common_vendor.onHide(() => {
      stopTransition();
    });
    function reset() {
      stopTransition();
      startTransition();
    }
    function startTransition() {
      common_vendor.nextTick$1(() => scroll());
    }
    function stopTransition() {
      transitionState.transitionProperty = "unset";
      transitionState.transitionDelay = "unset";
      transitionState.transitionDuration = "unset";
      transitionState.transform = "none";
      transitionState.transitionTimingFunction = "linear";
      currentIndex.value = 0;
      verticalIndex.value = 0;
    }
    function handleClose() {
      show.value = false;
      emit("close");
    }
    function setTransition({ duration, delay, translate }) {
      transitionState.transitionProperty = "all";
      transitionState.transitionDelay = `${delay}s`;
      transitionState.transitionDuration = `${duration}s`;
      transitionState.transform = `${props.direction === "vertical" ? "translateY" : "translateX"}(${translate}px)`;
      transitionState.transitionTimingFunction = "linear";
    }
    function queryRect() {
      return Promise.all([common_vendor.getRect($wrap, false, proxy), common_vendor.getRect($content, false, proxy)]);
    }
    async function verticalAnimate(height) {
      const translate = -(height / (textArray.value.length + 1)) * (currentIndex.value + 1);
      setTransition({
        duration: height / (textArray.value.length + 1) / props.speed,
        delay: props.delay,
        translate
      });
    }
    async function scroll() {
      const [wRect, cRect] = await queryRect();
      if (!wRect.width || !cRect.width || !cRect.height)
        return;
      wrapRect.value = wRect;
      contentRect.value = cRect;
      wrapWidth.value = wRect.width;
      if (isHorizontal.value) {
        if (props.scrollable) {
          setTransition({
            duration: cRect.width / props.speed,
            delay: props.delay,
            translate: -cRect.width
          });
        }
      } else {
        if (textArray.value.length > 1) {
          verticalAnimate(cRect.height);
        }
      }
    }
    function next() {
      if (currentIndex.value >= textArray.value.length - 1) {
        currentIndex.value = 0;
      } else {
        currentIndex.value++;
      }
      emit("next", currentIndex.value);
    }
    function animationEnd() {
      if (isHorizontal.value) {
        setTransition({
          duration: 0,
          delay: 0,
          translate: wrapWidth.value + 1
        });
      } else {
        if (++verticalIndex.value >= textArray.value.length) {
          verticalIndex.value = 0;
          setTransition({
            duration: 0,
            delay: 0,
            translate: 0
          });
        }
      }
      const timer = setTimeout(() => {
        next();
        common_vendor.nextTick$1(async () => {
          try {
            const [wRect, cRect] = await queryRect();
            wrapRect.value = wRect;
            contentRect.value = cRect;
            wrapWidth.value = wRect.width || 0;
          } catch (error) {
          }
          if (!contentRect.value || !contentRect.value.width || !contentRect.value.height)
            return;
          if (isHorizontal.value) {
            setTransition({
              duration: (wrapWidth.value + contentRect.value.width) / props.speed,
              delay: props.delay,
              translate: -contentRect.value.width
            });
          } else {
            verticalAnimate(contentRect.value.height);
          }
        });
        clearTimeout(timer);
      }, 20);
    }
    function handleClick() {
      const result = common_vendor.isArray(props.text) ? {
        index: currentIndex.value,
        text: props.text[currentIndex.value]
      } : {
        index: 0,
        text: props.text
      };
      emit("click", result);
    }
    __expose({ reset });
    return (_ctx, _cache) => {
      return common_vendor.e({
        a: show.value
      }, show.value ? common_vendor.e({
        b: _ctx.prefix
      }, _ctx.prefix ? {
        c: common_vendor.p({
          ["custom-class"]: "wd-notice-bar__prefix",
          name: _ctx.prefix
        })
      } : {}, {
        d: isVertical.value
      }, isVertical.value ? common_vendor.e({
        e: common_vendor.f(textArray.value, (item, k0, i0) => {
          return {
            a: common_vendor.t(item),
            b: item
          };
        }),
        f: textArray.value.length > 1
      }, textArray.value.length > 1 ? {
        g: common_vendor.t(textArray.value[0])
      } : {}) : {
        h: common_vendor.t(currentText.value)
      }, {
        i: common_vendor.s(animation.value),
        j: common_vendor.o(animationEnd, "8d"),
        k: common_vendor.o(handleClick, "ee"),
        l: _ctx.closable
      }, _ctx.closable ? {
        m: common_vendor.o(handleClose, "49"),
        n: common_vendor.p({
          ["custom-class"]: "wd-notice-bar__suffix",
          name: "close"
        })
      } : {}, {
        o: common_vendor.n(`wd-notice-bar ${_ctx.customClass} ${noticeBarClass.value}`),
        p: common_vendor.s(rootStyle.value)
      }) : {});
    };
  }
});
wx.createComponent(_sfc_main);
