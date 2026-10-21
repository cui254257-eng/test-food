import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import uniPlugin from '@dcloudio/vite-plugin-uni'
import { defineConfig } from 'vite'
import { WeappTailwindcss } from 'weapp-tailwindcss/vite'

const projectRoot = dirname(fileURLToPath(import.meta.url))
// The current DCloud package is CommonJS and may expose its factory one level
// below default when Vite loads this ESM config.
const createUniPlugins = typeof uniPlugin === 'function'
  ? uniPlugin
  : (uniPlugin as unknown as { default: typeof uniPlugin }).default

export default defineConfig({
  plugins: [
    createUniPlugins(),
    WeappTailwindcss({
      cssEntries: [resolve(projectRoot, 'src/app.css')],
      cssOptions: { rem2rpx: true },
    }),
  ],
  server: {
    host: '0.0.0.0',
    port: 5173,
    strictPort: true,
  },
  preview: {
    host: '0.0.0.0',
    port: 4173,
    strictPort: true,
  },
  css: {
    preprocessorOptions: {
      scss: {
        api: 'modern-compiler',
        silenceDeprecations: ['legacy-js-api'],
      },
    },
  },
})
