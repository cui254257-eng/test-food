# 好味小馆餐饮点餐

单门店餐饮点餐 MVP，包含微信小程序、门店管理后台、Rust API 和共享类型。支持菜品规格、堂食/自取/外卖选择、地址簿与服务端创建待支付订单。

## 技术栈

- 小程序：uni-app、Vue 3、TypeScript、Tailwind CSS 4、Wot UI
- 管理后台：React 19、TypeScript、Vite、Ant Design、React Router
- API：Rust、Axum、MySQL
- 包管理：pnpm workspace

## 环境要求

- Node.js 22.18 或更新版本
- pnpm 10.33.0
- Rust stable / Cargo
- MySQL（本地连接 `127.0.0.1:3307`，数据库 `shop`）
- 微信开发者工具（运行微信小程序时）

## 安装

```bash
pnpm install
```

本地 API 配置见 `.env.example`，复制为仓库根目录 `.env` 后填入微信小程序 AppID 和 Secret。本机的 MySQL 3307 实例可用 `powershell -File infra/mysql/start-shop-db.ps1` 启动，停止用 `powershell -File infra/mysql/stop-shop-db.ps1`。数据库 `shop` 需先存在；API 启动并连接数据库时会自动初始化所需的数据表。

微信登录、个人资料与收货地址使用同一顾客会话：`apps/mini/src/manifest.json` 中的 `mp-weixin.appid` 必须与 `.env` 的 `WECHAT_APP_ID` 一致；`WECHAT_APP_SECRET` 只放在 API 主机环境变量或未提交的 `.env`，不要放入小程序配置或提交到代码仓库。登录时由小程序取得一次性 `uni.login` code，后端换取 OpenID 并自动创建或复用顾客账号；客户端不能传入或指定用户 ID。登录 token 持久化保存在小程序本地和服务端数据库，有效期为 7 天。

登录后资料和地址 API 需 `Authorization: Bearer <token>`，登录接口除外：

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| POST | `/api/auth/wechat` | 使用小程序登录 code 自动注册或登录并创建会话 |
| GET | `/api/auth/session` | 校验当前微信用户的登录会话 |
| GET | `/api/auth/profile` | 获取当前用户昵称和头像 |
| PUT | `/api/auth/profile` | 修改当前用户昵称 |
| POST | `/api/auth/profile/avatar` | 上传并保存当前用户头像（multipart 字段 `avatar`，最大 1 MB） |
| POST | `/api/auth/logout` | 撤销当前服务端会话 |
| GET | `/api/addresses` | 获取当前微信用户的地址 |
| POST | `/api/addresses` | 新增当前用户的地址 |
| PUT | `/api/addresses/:id` | 修改当前用户的地址 |
| DELETE | `/api/addresses/:id` | 删除当前用户的地址 |

地址表不需要手动运行 SQL。`apps/api/src/main.rs` 启动时调用 `database::connect`，其内部在 `apps/api/src/database.rs` 的 `initialize_schema()` 中执行 `CREATE TABLE IF NOT EXISTS`：

- `customers`：微信用户映射及持久化昵称、头像（按 OpenID 自动创建或复用账号；客户端不会接触 OpenID）；新用户默认昵称为 `微信用户` 加数字后缀。
- `customer_sessions`：服务端会话 token 的哈希和过期时间，不保存明文 token；有效期 7 天。
- `customer_addresses`：收件人、电话、地区、详细地址及默认标记，外键关联到所属用户。
- `restaurant_settings`：单门店名称、公告、营业时段、服务开关和外卖配送参数。
- `restaurant_orders` / `restaurant_order_items`：订单主表与菜品快照；未支付订单 15 分钟后自动取消并释放已预留库存。
- `products`：当前兼作菜品表，新增 `category` 和 JSON `specifications` 字段；规格选项支持加价。

因此表会创建在 `DATABASE_URL` 指向的 MySQL 数据库/schema 中。默认开发配置是 MySQL `127.0.0.1:3307` 的 `shop` 数据库；首次运行前需要先启动 MySQL 并确保 `shop` 数据库已创建。部署时同样需给 API 配置正确的数据库和微信服务端凭证，再重启 API；当前项目通过启动时的建表语句初始化表，尚未使用独立 migration 工具。

## 启动

分别在终端运行以下命令：

```bash
# Rust API（默认 http://localhost:8000，首次启动会建表并写入示例菜品）
pnpm api

# React 管理后台（http://localhost:5173）
pnpm dev:admin

# uni-app H5（http://localhost:5173）
pnpm dev:mini:h5

# 微信小程序开发构建，完成后用微信开发者工具打开 apps/mini/dist/dev/mp-weixin
pnpm dev:mini:mp-weixin
```

管理后台和 H5 共用 Vite 默认端口 5173，请分开启动；微信小程序命令不启动 H5 页面服务。

## 门店与菜单

管理后台的“菜品菜单”可维护菜品、分类、售价、可售份数、图片和规格。小程序首页展示活动轮播图、堂食和点餐两个入口，以及可编辑公告；“我的”页启用微信登录，登录后可编辑并持久化头像和昵称。微信一键登录和手机号登录作为后续扩展入口暂不开放。

“门店设置”维护营业时间、当前接单状态、服务方式、门店地址和外卖规则；“首页公告”管理小程序首页展示的公告。点餐篮支持桌号、外带自取和外卖收货地址。

下单接口会在服务端重新读取菜价与规格、校验营业状态/服务方式/库存/起送金额，并在事务中创建订单、保存菜品快照及预留库存；相同请求键可安全重试。未支付订单 15 分钟后由 API 自动取消并释放库存。当前微信小程序端已接入，但微信支付、退款、后台接单和订单查询尚未接入，因此目前不能完成真实收款。配送半径目前仅用于展示，尚未使用地图坐标进行真实距离校验。

轮播图由管理后台上传：前景图和可选背景图分别转为 Data URL，POST 到 `/api/banners`；API 校验图片内容和格式后将 Base64 暂存到 `shop.banners`。每张图片最大 5 MB，支持 JPG、PNG、WEBP、GIF。小程序读取轮播图列表，并用独立背景层跟随滑动进度交叉渐变；未配置背景图的旧轮播会继续使用前景图铺底。后续接入对象存储时，只需替换 Rust 的 `BannersRepository` 存储实现。

API 路由：

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/health` | 服务健康检查 |
| GET | `/api/products` | 菜品列表，可传 `q` 和 `status` |
| GET | `/api/products/:id` | 菜品详情（含分类与规格） |
| POST | `/api/products` | 新增菜品 |
| PUT | `/api/products/:id` | 更新菜品 |
| DELETE | `/api/products/:id` | 删除菜品 |
| GET/PUT | `/api/restaurant/settings` | 查看/更新单门店营业、服务方式与配送配置 |
| GET/PUT | `/api/announcements` | 查看/更新小程序首页公告 |
| POST | `/api/orders` | 登录用户提交点餐订单；菜价由服务端计算 |
| GET | `/api/banners` | 门店活动图列表 |
| POST | `/api/banners` | 上传活动图，JSON 字段 `title`、`image_data`（Data URL）、可选 `background_image_data` |
| PUT | `/api/banners/:id` | 更新标题、排序值和/或前景图、背景图 |
| GET | `/api/banners/:id/image` | 获取活动图原图 |
| GET | `/api/banners/:id/background-image` | 获取可选背景图 |
| DELETE | `/api/banners/:id` | 删除活动图 |

顾客订单接口需要微信登录令牌。管理后台当前没有登录鉴权，已有菜单、公告和门店配置写接口不能直接暴露到公网；上线前必须先接入管理员身份、角色权限和审计日志。

## API 地址

API 默认监听 `0.0.0.0:8000`。可以在 `apps/admin/.env.local` 或 `apps/mini/.env.local` 设置 `VITE_API_BASE_URL`。真机或微信开发者工具连接时，将小程序地址改为开发机局域网 IP；发布前需配置 HTTPS 域名及小程序合法请求域名。

## 构建与类型检查

```bash
pnpm build:admin
pnpm typecheck:admin
pnpm typecheck:mini
pnpm build:mini:h5
pnpm build:mini:mp-weixin
cargo check
```

## 目录结构

```text
apps/
  admin/       React 管理后台
  api/         Rust API
    src/
      main.rs                  启动入口，对应 NestJS main.ts
      app.rs                   组装全局路由和依赖，对应 AppModule
      config.rs                环境配置
      database.rs              数据库初始化和示例数据
      common/                  通用错误处理
      modules/
        banners/               轮播图接口、存储和图片响应
        auth/                  微信登录和服务端用户会话
        addresses/             用户隔离的收货地址簿
        products/
          products_module.rs   菜单数据接口（兼容沿用 /api/products 路由）
          controller.rs        HTTP 接口，对应 ProductsController
          service.rs           业务校验和流程，对应 ProductsService
          repository.rs        SQL 查询和持久化
          entity.rs            菜品数据模型
          dto.rs               请求参数结构
      assets/icons/            小程序首页八个 SVG 图标
  mini/        uni-app 小程序
packages/
  contracts/   前后端共享的菜品与规格类型
```

Rust API 按业务模块放代码。新增业务时，在 `apps/api/src/modules/` 下新建模块，并把路由、接口、业务逻辑、数据访问和数据结构放在同一目录；`main.rs` 只负责启动，跨模块通用代码放在 `common/`。
