import { AppstoreOutlined, BellOutlined, PictureOutlined, SettingOutlined, ShopOutlined } from '@ant-design/icons'
import { Layout, Menu, Typography } from 'antd'
import { Link, Outlet, useLocation } from 'react-router-dom'

const { Sider, Content } = Layout

export default function AdminLayout() {
  const location = useLocation()
  const selectedKey = location.pathname.startsWith('/products')
    ? '/products'
    : location.pathname.startsWith('/banners')
      ? '/banners'
      : location.pathname.startsWith('/announcements')
        ? '/announcements'
        : location.pathname.startsWith('/restaurant')
          ? '/restaurant'
      : '/dashboard'

  return (
    <Layout className="admin-shell">
      <Sider breakpoint="lg" collapsedWidth="0" className="admin-sider">
        <Link to="/dashboard" className="brand-lockup">
          <span className="brand-mark">S</span>
          <span>
            <strong>HAO WEI</strong>
            <small>管理后台</small>
          </span>
        </Link>
        <Menu
          theme="dark"
          mode="inline"
          selectedKeys={[selectedKey]}
          items={[
            { key: '/dashboard', icon: <AppstoreOutlined />, label: <Link to="/dashboard">概览</Link> },
            { key: '/products', icon: <ShopOutlined />, label: <Link to="/products">菜品菜单</Link> },
            { key: '/banners', icon: <PictureOutlined />, label: <Link to="/banners">门店活动图</Link> },
            { key: '/announcements', icon: <BellOutlined />, label: <Link to="/announcements">首页公告</Link> },
            { key: '/restaurant', icon: <SettingOutlined />, label: <Link to="/restaurant">门店设置</Link> },
          ]}
        />
      </Sider>
      <Layout>
        <div className="admin-topbar">
          <Typography.Text type="secondary">好味小馆 · 门店管理</Typography.Text>
          <div className="admin-user"><span className="user-dot" />管理员</div>
        </div>
        <Content className="admin-content">
          <Outlet />
        </Content>
      </Layout>
    </Layout>
  )
}
