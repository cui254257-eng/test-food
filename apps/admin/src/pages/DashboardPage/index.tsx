import { useEffect, useState } from 'react'
import { Alert, Card, Col, Row, Space, Statistic, Typography } from 'antd'
import { getProducts } from '@/api/products'

export default function DashboardPage() {
  const [stats, setStats] = useState<{ total: number; active: number }>()
  const [error, setError] = useState('')

  useEffect(() => {
    getProducts()
      .then((products) => setStats({
        total: products.length,
        active: products.filter((product) => product.status === 'active' && product.stock > 0).length,
      }))
      .catch(() => setError('无法连接菜品服务'))
  }, [])

  return (
    <Space direction="vertical" size={24} className="page-stack">
      <div className="page-heading">
        <Typography.Title level={2}>门店概况</Typography.Title>
      </div>
      {error && <Alert type="warning" showIcon message={error} />}
      <Row gutter={[16, 16]}>
        <Col xs={24} md={8}>
          <Card className="stat-card"><Statistic title="菜品总数" value={stats?.total ?? '—'} suffix="款" /></Card>
        </Col>
        <Col xs={24} md={8}>
          <Card className="stat-card"><Statistic title="当前可点" value={stats?.active ?? '—'} suffix="款" /></Card>
        </Col>
        <Col xs={24} md={8}>
          <Card className="stat-card"><Statistic title="菜单服务" value={error ? '异常' : stats ? '正常' : '检查中'} /></Card>
        </Col>
      </Row>
    </Space>
  )
}
