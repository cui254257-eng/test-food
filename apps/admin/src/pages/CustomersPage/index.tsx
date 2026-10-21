import { useEffect, useState } from "react";
import {
  Alert,
  Button,
  Card,
  Empty,
  Input,
  Pagination,
  Table,
  Tag,
  Typography,
} from "antd";
import type { TableProps } from "antd";
import { getAdminCustomers, type AdminCustomer } from "@/api/membership";

const currency = new Intl.NumberFormat("zh-CN", {
  style: "currency",
  currency: "CNY",
});

export default function CustomersPage() {
  const [customers, setCustomers] = useState<AdminCustomer[]>([]);
  const [query, setQuery] = useState("");
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [refreshKey, setRefreshKey] = useState(0);

  useEffect(() => {
    let active = true;
    setLoading(true);
    setError("");
    getAdminCustomers({ q: search, page, page_size: pageSize })
      .then((result) => {
        if (!active) return;
        setCustomers(result.customers);
        setTotal(result.total);
      })
      .catch((reason: unknown) => {
        if (active)
          setError(reason instanceof Error ? reason.message : "用户加载失败");
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [search, page, pageSize, refreshKey]);

  const columns: TableProps<AdminCustomer>["columns"] = [
    {
      title: "用户",
      dataIndex: "nickname",
      render: (nickname: string) => (
        <>
          <Typography.Text strong>{nickname || "微信用户"}</Typography.Text>
        </>
      ),
    },
    {
      title: "等级",
      dataIndex: "member_level",
      width: 90,
      render: (level: number) => (
        <Tag color={level > 1 ? "gold" : "default"}>V{level}</Tag>
      ),
    },
    { title: "积分", dataIndex: "points", width: 110 },
    { title: "累计订单", dataIndex: "order_count", width: 110 },
    {
      title: "累计实付",
      dataIndex: "paid_total_cents",
      width: 140,
      render: (cents: number) => currency.format(cents / 100),
    },
    { title: "注册时间", dataIndex: "created_at", width: 175 },
  ];

  return (
    <div className="page-stack">
      <div className="page-heading">
        <Typography.Title level={2}>用户列表</Typography.Title>
        <Button onClick={() => setRefreshKey((key) => key + 1)}>刷新</Button>
      </div>
      {error && <Alert type="error" showIcon message={error} />}
      <Card className="stat-card">
        <Input.Search
          allowClear
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          onSearch={(value) => {
            setPage(1);
            setSearch(value.trim());
          }}
          placeholder="搜索昵称或用户 ID"
          style={{ width: 300 }}
        />
      </Card>
      <Card className="stat-card" title={`用户 · ${total}`}>
        <Table<AdminCustomer>
          rowKey="id"
          className="products-table"
          columns={columns}
          dataSource={customers}
          loading={loading}
          pagination={false}
          locale={{ emptyText: <Empty description="暂无用户" /> }}
          scroll={{ x: 760 }}
        />
        <Pagination
          current={page}
          pageSize={pageSize}
          total={total}
          showSizeChanger
          pageSizeOptions={[10, 20, 50, 100]}
          showTotal={(count) => `共 ${count} 人`}
          onChange={(nextPage, nextSize) => {
            setPage(nextPage);
            setPageSize(nextSize);
          }}
          style={{ display: "flex", justifyContent: "flex-end", marginTop: 18 }}
        />
      </Card>
    </div>
  );
}
