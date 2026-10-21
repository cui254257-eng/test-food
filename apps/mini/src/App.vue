<script lang="ts">
export default {
  onLaunch() {
    console.info('Shop mini app launched')
  },
}
</script>

<style>
@import './app.css';
</style>

<style lang="scss">
@use '@wot-ui/ui/styles/theme/index.scss' as *;

page {
  min-height: 100%;
  background: #f6f7f9;
  color: #1f2937;
  font-family: -apple-system, BlinkMacSystemFont, 'PingFang SC', sans-serif;
}
</style>
