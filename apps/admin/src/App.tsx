import { lazy, Suspense, type ReactNode } from "react";
import { Spin } from "antd";
import { Navigate, Route, Routes } from "react-router-dom";
import AdminLayout from "./components/AdminLayout";

const DashboardPage = lazy(() => import("./pages/DashboardPage"));
const ProductsPage = lazy(() => import("./pages/ProductsPage"));
const BannersPage = lazy(() => import("./pages/BannersPage"));
const RestaurantPage = lazy(() => import("./pages/RestaurantPage"));
const AnnouncementPage = lazy(() => import("./pages/AnnouncementPage"));

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

export default function App() {
  return (
    <Routes>
      <Route element={<AdminLayout />}>
        <Route index element={<Navigate to="/dashboard" replace />} />
        <Route path="dashboard" element={loadPage(<DashboardPage />)} />
        <Route path="products" element={loadPage(<ProductsPage />)} />
        <Route path="banners" element={loadPage(<BannersPage />)} />
        <Route path="restaurant" element={loadPage(<RestaurantPage />)} />
        <Route path="announcements" element={loadPage(<AnnouncementPage />)} />
        <Route path="*" element={<Navigate to="/dashboard" replace />} />
      </Route>
    </Routes>
  );
}
