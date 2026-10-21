import { useEffect, useState } from "react";
import {
  Alert,
  Button,
  Card,
  Descriptions,
  Empty,
  message,
  Pagination,
  Popconfirm,
  Select,
  Space,
  Table,
  Tag,
  Tooltip,
  Typography,
} from "antd";
import type { TableProps } from "antd";
import { getAdminToken } from "@/api/adminAuth";
import {
  deleteAdminOrder,
  getAdminOrders,
  markAdminOrderPaid,
  type AdminOrder,
  type AdminOrderFilters,
} from "@/api/orders";
import { getAdminCustomers, type AdminCustomer } from "@/api/membership";

const currency = new Intl.NumberFormat("zh-CN", {
  style: "currency",
  currency: "CNY",
});

function money(cents: number) {
  return currency.format(cents / 100);
}

function modeLabel(mode: AdminOrder["fulfillment_mode"]) {
  return mode === "delivery"
    ? "外卖配送"
    : mode === "dine_in"
      ? "堂食"
      : "到店自取";
}

function statusView(status: string) {
  const known: Record<string, { label: string; color: string }> = {
    pending_payment: { label: "待付款", color: "orange" },
    paid: { label: "已支付", color: "blue" },
    preparing: { label: "制作中", color: "processing" },
    ready: { label: "待取餐", color: "cyan" },
    completed: { label: "已完成", color: "green" },
    cancelled: { label: "已取消", color: "default" },
  };
  return known[status] ?? { label: status || "处理中", color: "default" };
}

const columns: NonNullable<TableProps<AdminOrder>["columns"]> = [
  {
    title: "订单信息",
    dataIndex: "order_number",
    width: 240,
    render: (_, order) => (
      <Space direction="vertical" size={2}>
        <Typography.Text strong copyable>
          {order.order_number}
        </Typography.Text>
        <Typography.Text type="secondary">{order.created_at}</Typography.Text>
      </Space>
    ),
  },
  {
    title: "下单用户",
    dataIndex: "customer_name",
    width: 155,
    render: (_, order) => (
      <Space direction="vertical" size={2}>
        <Typography.Text>{order.customer_name || "微信用户"}</Typography.Text>
      </Space>
    ),
  },
  {
    title: "商品",
    dataIndex: "items",
    ellipsis: true,
    render: (items: AdminOrder["items"]) =>
      items
        .map((item) => `${item.product_name} ×${item.quantity}`)
        .join("、") || "—",
  },
  {
    title: "用餐方式",
    dataIndex: "fulfillment_mode",
    width: 120,
    render: (mode: AdminOrder["fulfillment_mode"]) => modeLabel(mode),
  },
  {
    title: "订单金额",
    dataIndex: "total_cents",
    width: 120,
    align: "right",
    render: (cents: number) => (
      <Typography.Text strong>{money(cents)}</Typography.Text>
    ),
  },
  {
    title: "状态",
    dataIndex: "status",
    width: 105,
    render: (status: string) => {
      const view = statusView(status);
      return <Tag color={view.color}>{view.label}</Tag>;
    },
  },
];

export default function OrdersPage() {
  const [token] = useState(getAdminToken);
  const [orders, setOrders] = useState<AdminOrder[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [deletingOrder, setDeletingOrder] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [customers, setCustomers] = useState<AdminCustomer[]>([]);
  const [selectedCustomer, setSelectedCustomer] =
    useState<AdminCustomer | null>(null);
  const [customerInput, setCustomerInput] = useState<number | undefined>();
  const [customerSearch, setCustomerSearch] = useState("");
  const [customersLoading, setCustomersLoading] = useState(false);
  const [customersError, setCustomersError] = useState("");
  const [statusInput, setStatusInput] = useState("");
  const [modeInput, setModeInput] = useState("");
  const [filters, setFilters] = useState<
    Omit<AdminOrderFilters, "page" | "page_size">
  >({});
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [refreshKey, setRefreshKey] = useState(0);

  useEffect(() => {
    let active = true;
    const timeout = window.setTimeout(() => {
      setCustomersLoading(true);
      setCustomersError("");
      getAdminCustomers({
        q: customerSearch.trim() || undefined,
        page: 1,
        page_size: 100,
      })
        .then((result) => {
          if (active) setCustomers(result.customers);
        })
        .catch((requestError: unknown) => {
          if (active)
            setCustomersError(
              requestError instanceof Error
                ? requestError.message
                : "用户加载失败",
            );
        })
        .finally(() => {
          if (active) setCustomersLoading(false);
        });
    }, 250);
    return () => {
      active = false;
      window.clearTimeout(timeout);
    };
  }, [customerSearch, refreshKey]);

  useEffect(() => {
    if (!token) return;
    let active = true;
    setLoading(true);
    setError("");
    getAdminOrders({ ...filters, page, page_size: pageSize })
      .then((result) => {
        if (!active) return;
        setOrders(result.orders);
        setTotal(result.total);
      })
      .catch((requestError: unknown) => {
        if (active)
          setError(
            requestError instanceof Error
              ? requestError.message
              : "订单加载失败",
          );
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [token, filters, page, pageSize, refreshKey]);

  function applyFilters() {
    setFilters({
      customer_id: customerInput,
      status: statusInput || undefined,
      fulfillment_mode: modeInput || undefined,
    });
    setPage(1);
  }

  async function removeOrder(order: AdminOrder) {
    setDeletingOrder(order.order_number);
    try {
      await deleteAdminOrder(order.order_number);
      message.success(`订单 ${order.order_number} 已删除`);
      if (orders.length === 1 && page > 1) setPage((current) => current - 1);
      setRefreshKey((value) => value + 1);
    } catch (reason) {
      message.error(reason instanceof Error ? reason.message : "删除订单失败");
    } finally {
      setDeletingOrder(null);
    }
  }

  const orderColumns: NonNullable<TableProps<AdminOrder>["columns"]> = [
    ...columns,
    {
      title: "操作",
      dataIndex: "order_number",
      width: 90,
      render: (_, order) => {
        const deleteButton = (
          <Button
            type="link"
            danger
            disabled={order.payment_status === "paid"}
            loading={deletingOrder === order.order_number}
          >
            删除
          </Button>
        );
        if (order.payment_status === "paid") {
          return (
            <Tooltip title="已支付订单涉及财务和积分记录，不能直接删除">
              <span>{deleteButton}</span>
            </Tooltip>
          );
        }
        return (
          <Popconfirm
            title="确认删除这笔订单？"
            description="删除后无法恢复；如库存尚未释放，系统会自动归还库存。"
            okText="确认删除"
            cancelText="取消"
            okButtonProps={{ danger: true }}
            onConfirm={() => removeOrder(order)}
          >
            {deleteButton}
          </Popconfirm>
        );
      },
    },
  ];

  return (
    <div className="page-stack">
      {error && (
        <Alert
          type="error"
          showIcon
          message={error}
          action={
            <Button
              size="small"
              onClick={() => setRefreshKey((value) => value + 1)}
            >
              重试
            </Button>
          }
        />
      )}
      {customersError && (
        <Alert
          type="error"
          showIcon
          message={`用户列表加载失败：${customersError}`}
        />
      )}

      <Card className="stat-card">
        <Space wrap size={[10, 12]}>
          <Select
            showSearch
            allowClear
            filterOption={false}
            value={customerInput}
            loading={customersLoading}
            onSearch={setCustomerSearch}
            onChange={(value: number | undefined) => {
              setCustomerInput(value);
              if (value === undefined) {
                setSelectedCustomer(null);
              } else {
                setSelectedCustomer(
                  customers.find((customer) => customer.id === value) ??
                    selectedCustomer,
                );
                setCustomerSearch("");
              }
            }}
            placeholder="选择用户（搜索昵称）"
            style={{ width: 260 }}
            notFoundContent={
              customersLoading
                ? "正在加载用户…"
                : customersError || "没有匹配的用户"
            }
            options={[
              ...(selectedCustomer &&
              !customers.some((customer) => customer.id === selectedCustomer.id)
                ? [
                    {
                      value: selectedCustomer.id,
                      label: `${selectedCustomer.nickname || "微信用户"} · #${selectedCustomer.id}`,
                    },
                  ]
                : []),
              ...customers.map((customer) => ({
                value: customer.id,
                label: `${customer.nickname || "微信用户"} · #${customer.id}`,
              })),
            ]}
          />
          <Select
            value={statusInput || undefined}
            onChange={(value) => setStatusInput(value ?? "")}
            allowClear
            placeholder="全部订单状态"
            style={{ width: 150 }}
            options={[
              { value: "pending_payment", label: "待付款" },
              { value: "paid", label: "已支付" },
              { value: "preparing", label: "制作中" },
              { value: "ready", label: "待取餐" },
              { value: "completed", label: "已完成" },
              { value: "cancelled", label: "已取消" },
            ]}
          />
          <Select
            value={modeInput || undefined}
            onChange={(value) => setModeInput(value ?? "")}
            allowClear
            placeholder="全部用餐方式"
            style={{ width: 150 }}
            options={[
              { value: "dine_in", label: "堂食" },
              { value: "takeaway", label: "到店自取" },
              { value: "delivery", label: "外卖配送" },
            ]}
          />
          <Button type="primary" onClick={applyFilters}>
            筛选
          </Button>
          <Button
            onClick={() => {
              setCustomerInput(undefined);
              setSelectedCustomer(null);
              setCustomerSearch("");
              setStatusInput("");
              setModeInput("");
              setFilters({});
              setPage(1);
            }}
          >
            重置
          </Button>
        </Space>
      </Card>

      <Card className="stat-card" title={`订单列表 · 共 ${total} 笔`}>
        <Table<AdminOrder>
          rowKey="order_number"
          className="products-table"
          columns={orderColumns}
          dataSource={orders}
          loading={loading}
          scroll={{ x: 1040 }}
          locale={{ emptyText: <Empty description="没有符合条件的订单" /> }}
          expandable={{
            expandedRowRender: (order) => (
              <div style={{ display: "grid", gap: 16, padding: "8px 4px" }}>
                <Descriptions size="small" column={{ xs: 1, sm: 2, md: 3 }}>
                  <Descriptions.Item label="支付状态">
                    {order.payment_status === "paid" ? "已支付" : "未支付"}
                  </Descriptions.Item>
                  <Descriptions.Item label="商品小计">
                    {money(order.subtotal_cents)}
                  </Descriptions.Item>
                  <Descriptions.Item label="会员优惠">
                    -{money(order.discount_cents)}
                  </Descriptions.Item>
                  <Descriptions.Item label="配送费">
                    {money(order.delivery_fee_cents)}
                  </Descriptions.Item>
                  <Descriptions.Item label="本单积分">
                    {order.points_earned}
                  </Descriptions.Item>
                  {order.table_number && (
                    <Descriptions.Item label="桌号">
                      {order.table_number}
                    </Descriptions.Item>
                  )}
                  {order.delivery_recipient_name && (
                    <Descriptions.Item label="收件人">
                      {order.delivery_recipient_name}
                    </Descriptions.Item>
                  )}
                  {order.delivery_phone && (
                    <Descriptions.Item label="联系电话">
                      {order.delivery_phone}
                    </Descriptions.Item>
                  )}
                  {order.delivery_address && (
                    <Descriptions.Item label="配送地址">
                      {order.delivery_address}
                    </Descriptions.Item>
                  )}
                  {order.status === "pending_payment" && (
                    <Descriptions.Item label="待支付截止">
                      {order.expires_at}
                    </Descriptions.Item>
                  )}
                </Descriptions>
                <div className="order-detail-lines">
                  <Typography.Text strong>商品明细</Typography.Text>
                  {order.items.map((item, index) => (
                    <div
                      className="order-detail-line"
                      key={`${item.product_name}-${index}`}
                    >
                      <span>
                        {item.product_name}
                        {item.selected_options.length > 0 &&
                          `（${item.selected_options.map((option) => option.option_name).join("、")}）`}
                      </span>
                      <span>× {item.quantity}</span>
                      <span>{money(item.line_total_cents)}</span>
                    </div>
                  ))}
                </div>
                {order.status === "pending_payment" && (
                  <Popconfirm
                    title="确认已收款？"
                    description="确认后将按实付商品金额发放会员积分。"
                    onConfirm={async () => {
                      try {
                        const result = await markAdminOrderPaid(
                          order.order_number,
                        );
                        message.success(
                          `已确认收款，发放 ${result.points_earned} 积分`,
                        );
                        setRefreshKey((value) => value + 1);
                      } catch (reason) {
                        message.error(
                          reason instanceof Error ? reason.message : "操作失败",
                        );
                      }
                    }}
                  >
                    <Button type="primary" size="small">
                      确认收款并发放积分
                    </Button>
                  </Popconfirm>
                )}
              </div>
            ),
          }}
          pagination={false}
        />
        <Pagination
          current={page}
          pageSize={pageSize}
          total={total}
          showSizeChanger
          pageSizeOptions={[10, 20, 50, 100]}
          showTotal={(count) => `共 ${count} 笔`}
          onChange={(nextPage, nextPageSize) => {
            setPage(nextPage);
            setPageSize(nextPageSize);
          }}
          style={{ display: "flex", justifyContent: "flex-end", marginTop: 18 }}
        />
      </Card>
    </div>
  );
}
