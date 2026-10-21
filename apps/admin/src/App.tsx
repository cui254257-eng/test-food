import { lazy, Suspense, useEffect, useState, type ReactNode } from "react";
import { Spin } from "antd";
import { Navigate, Route, Routes, useLocation } from "react-router-dom";
import AdminLayout from "./components/AdminLayout";
import { Alert, Button } from "antd";
import { getAdminToken, verifyAdminSession } from "./api/adminAuth";
import { ADMIN_AUTH_EXPIRED_EVENT } from "./api/adminToken";

const LoginPage = lazy(() => import("./pages/LoginPage"));
const DashboardPage = lazy(() => import("./pages/DashboardPage"));
const ProductsPage = lazy(() => import("./pages/ProductsPage"));
const BannersPage = lazy(() => import("./pages/BannersPage"));
const RestaurantPage = lazy(() => import("./pages/RestaurantPage"));
const AnnouncementPage = lazy(() => import("./pages/AnnouncementPage"));
const OrdersPage = lazy(() => import("./pages/OrdersPage"));
const MembershipPage = lazy(() => import("./pages/MembershipPage"));
const CustomersPage = lazy(() => import("./pages/CustomersPage"));

function loadPage(page: ReactNode) {
  return (
    <Suspense
      fallback={
        <div className="route-loading">
          <Spin />
        </div>
      }
    >
      {page}
    </Suspense>
  );
}

function RequireAdmin({ children }: { children: ReactNode }) {
  const location = useLocation()
  const token = getAdminToken()
  const [retryKey, setRetryKey] = useState(0)
  const [status, setStatus] = useState<'checking' | 'authenticated' | 'unauthenticated' | 'error'>(
    token ? 'checking' : 'unauthenticated',
  )

  useEffect(() => {
    const onAuthExpired = () => setStatus('unauthenticated')
    const onStorageChange = (event: StorageEvent) => {
      if (event.key === 'restaurant-admin-api-token' && !event.newValue) {
        setStatus('unauthenticated')
      }
    }
    window.addEventListener(ADMIN_AUTH_EXPIRED_EVENT, onAuthExpired)
    window.addEventListener('storage', onStorageChange)
    if (!token) {
      setStatus('unauthenticated')
      return () => {
        window.removeEventListener(ADMIN_AUTH_EXPIRED_EVENT, onAuthExpired)
        window.removeEventListener('storage', onStorageChange)
      }
    }
    let active = true
    verifyAdminSession()
      .then((authenticated) => {
        if (active) setStatus(authenticated ? 'authenticated' : 'unauthenticated')
      })
      .catch(() => {
        // Preserve a valid token during API/network outages; only a 401 clears it.
        if (active) setStatus(getAdminToken() ? 'error' : 'unauthenticated')
      })
    return () => {
      active = false
      window.removeEventListener(ADMIN_AUTH_EXPIRED_EVENT, onAuthExpired)
      window.removeEventListener('storage', onStorageChange)
    }
  }, [token, retryKey])

  if (status === 'checking') {
    return <div className="route-loading"><Spin /></div>
  }
  if (status === 'error') {
    return (
      <div className="route-loading" style={{ flexDirection: 'column', gap: 16 }}>
        <Alert type="error" showIcon message="暂时无法连接服务器，登录状态已保留。" />
        <Button onClick={() => { setStatus('checking'); setRetryKey((key) => key + 1) }}>重试</Button>
      </div>
    )
  }
  if (status !== 'authenticated') {
    return <Navigate to="/login" replace state={{ from: location }} />
  }
  return children
}

export default function App() {
  return (
    <Routes>
      <Route path="login" element={loadPage(<LoginPage />)} />
      <Route element={<RequireAdmin><AdminLayout /></RequireAdmin>}>
        <Route index element={<Navigate to="/dashboard" replace />} />
        <Route path="dashboard" element={loadPage(<DashboardPage />)} />
        <Route path="products" element={loadPage(<ProductsPage />)} />
        <Route path="banners" element={loadPage(<BannersPage />)} />
        <Route path="restaurant" element={loadPage(<RestaurantPage />)} />
        <Route path="announcements" element={loadPage(<AnnouncementPage />)} />
        <Route path="orders" element={loadPage(<OrdersPage />)} />
        <Route path="membership" element={loadPage(<MembershipPage />)} />
        <Route path="customers" element={loadPage(<CustomersPage />)} />
      </Route>
      <Route path="*" element={<Navigate to="/dashboard" replace />} />
    </Routes>
  );
}
