import { useEffect, useMemo, useRef, useState } from 'react'
import * as echarts from 'echarts/core'
import type { EChartsOption, EChartsType } from 'echarts'
import { LineChart, PieChart } from 'echarts/charts'
import { GraphicComponent, GridComponent, LegendComponent, TooltipComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import { Alert, Card, Col, Row, Statistic } from 'antd'
import { getAdminOrders, type AdminOrder } from '@/api/orders'
import { getProducts } from '@/api/products'

echarts.use([LineChart, PieChart, GraphicComponent, GridComponent, LegendComponent, TooltipComponent, CanvasRenderer])

const modeNames: Record<AdminOrder['fulfillment_mode'], string> = {
  dine_in: '堂食',
  takeaway: '自取',
  delivery: '外卖',
}

const statusNames: Record<string, string> = {
  pending_payment: '待付款',
  paid: '已支付',
  preparing: '制作中',
  ready: '待取餐',
  completed: '已完成',
  cancelled: '已取消',
}

function dateKey(date: Date) {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

function orderDateKey(value: string) {
  return value.slice(0, 10)
}

function Chart({ option }: { option: EChartsOption }) {
  const elementRef = useRef<HTMLDivElement>(null)
  const chartRef = useRef<EChartsType | null>(null)

  useEffect(() => {
    if (!elementRef.current) return
    const chart = echarts.init(elementRef.current)
    chartRef.current = chart
    const observer = new ResizeObserver(() => chart.resize())
    observer.observe(elementRef.current)

    return () => {
      observer.disconnect()
      chart.dispose()
      chartRef.current = null
    }
  }, [])

  useEffect(() => {
    chartRef.current?.setOption(option, true)
  }, [option])

  return <div ref={elementRef} className="dashboard-chart" />
}

export default function DashboardPage() {
  const [orders, setOrders] = useState<AdminOrder[]>([])
  const [orderTotal, setOrderTotal] = useState<number>()
  const [productStats, setProductStats] = useState<{ total: number; active: number }>()
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    let active = true
    Promise.allSettled([
      getAdminOrders({ page: 1, page_size: 100 }),
      getProducts(),
    ] as const)
      .then(([orderResult, productResult]) => {
        if (!active) return
        const errors: string[] = []
        if (orderResult.status === 'fulfilled') {
          setOrders(orderResult.value.orders)
          setOrderTotal(orderResult.value.total)
        }
        else errors.push(orderResult.reason instanceof Error ? orderResult.reason.message : '订单数据加载失败')
        if (productResult.status === 'fulfilled') {
          setProductStats({
            total: productResult.value.length,
            active: productResult.value.filter((product) => product.status === 'active' && product.stock > 0).length,
          })
        } else errors.push('菜品数据加载失败')
        setError(errors.join(' · '))
      })
      .finally(() => {
        if (active) setLoading(false)
      })
    return () => { active = false }
  }, [])

  const paidAmount = useMemo(
    () => orders.reduce((sum, order) => sum + (order.payment_status === 'paid' ? order.total_cents : 0), 0) / 100,
    [orders],
  )

  const trendOption = useMemo<EChartsOption>(() => {
    const dates = Array.from({ length: 7 }, (_, index) => {
      const date = new Date()
      date.setDate(date.getDate() - (6 - index))
      return date
    })
    const keys = dates.map(dateKey)
    const orderCounts = new Array(7).fill(0) as number[]
    const paidAmounts = new Array(7).fill(0) as number[]

    orders.forEach((order) => {
      const index = keys.indexOf(orderDateKey(order.created_at))
      if (index < 0) return
      orderCounts[index] += 1
      if (order.payment_status === 'paid') paidAmounts[index] += order.total_cents / 100
    })

    return {
      color: ['#5b8ff9', '#61dDAa'],
      tooltip: { trigger: 'axis' },
      legend: { bottom: 0, itemWidth: 9, itemHeight: 9, textStyle: { color: '#778397' } },
      grid: { top: 22, right: 45, bottom: 48, left: 42 },
      xAxis: {
        type: 'category',
        boundaryGap: false,
        data: dates.map((date) => `${date.getMonth() + 1}/${date.getDate()}`),
        axisLine: { lineStyle: { color: '#e9edf3' } },
        axisTick: { show: false },
        axisLabel: { color: '#8994a4' },
      },
      yAxis: [
        { type: 'value', minInterval: 1, splitLine: { lineStyle: { color: '#f0f2f5' } }, axisLabel: { color: '#8994a4' } },
        { type: 'value', splitLine: { show: false }, axisLabel: { color: '#8994a4', formatter: '¥{value}' } },
      ],
      series: [
        {
          name: '订单',
          type: 'line',
          smooth: true,
          symbolSize: 6,
          data: orderCounts,
          areaStyle: { opacity: 0.08 },
        },
        {
          name: '实收',
          type: 'line',
          yAxisIndex: 1,
          smooth: true,
          symbolSize: 6,
          data: paidAmounts,
        },
      ],
    }
  }, [orders])

  const modeOption = useMemo<EChartsOption>(() => {
    const counts = new Map<string, number>()
    orders.forEach((order) => counts.set(order.fulfillment_mode, (counts.get(order.fulfillment_mode) ?? 0) + 1))
    const data = Object.entries(modeNames).map(([key, name]) => ({ name, value: counts.get(key) ?? 0 }))

    return {
      color: ['#5b8ff9', '#61dDAa', '#f6bd16'],
      tooltip: { trigger: 'item', formatter: '{b}：{c} 笔（{d}%）' },
      legend: { bottom: 0, itemWidth: 9, itemHeight: 9, textStyle: { color: '#778397' } },
      graphic: orders.length === 0 ? [{ type: 'text', left: 'center', top: 'middle', style: { text: '暂无订单', fill: '#a0a8b5', fontSize: 13 } }] : [],
      series: [{
        name: '用餐方式',
        type: 'pie',
        radius: ['48%', '72%'],
        center: ['50%', '46%'],
        avoidLabelOverlap: true,
        itemStyle: { borderColor: '#fff', borderWidth: 3, borderRadius: 5 },
        label: { color: '#657286', formatter: '{b}\n{d}%' },
        labelLine: { length: 10, length2: 8, lineStyle: { color: '#c8d0da' } },
        data,
      }],
    }
  }, [orders])

  const statusOption = useMemo<EChartsOption>(() => {
    const counts = new Map<string, number>()
    orders.forEach((order) => counts.set(order.status, (counts.get(order.status) ?? 0) + 1))
    const statuses = Object.keys(statusNames).filter((status) => counts.has(status))
    const colors = ['#f6bd16', '#5b8ff9', '#9270ca', '#5ad8a6', '#5d7092', '#e8684a']

    return {
      tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
      grid: { top: 12, right: 24, bottom: 18, left: 78, containLabel: false },
      xAxis: { type: 'value', minInterval: 1, splitLine: { lineStyle: { color: '#f0f2f5' } }, axisLabel: { color: '#8994a4' } },
      yAxis: { type: 'category', inverse: true, data: statuses.map((status) => statusNames[status] ?? status), axisTick: { show: false }, axisLine: { show: false }, axisLabel: { color: '#657286' } },
      series: [{
        type: 'bar',
        barMaxWidth: 22,
        data: statuses.map((status, index) => ({ value: counts.get(status) ?? 0, itemStyle: { color: colors[index % colors.length], borderRadius: [0, 6, 6, 0] } })),
        label: { show: true, position: 'right', color: '#657286' },
      }],
      graphic: orders.length === 0 ? [{ type: 'text', left: 'center', top: 'middle', style: { text: '暂无订单', fill: '#a0a8b5', fontSize: 13 } }] : [],
    }
  }, [orders])

  const popularOption = useMemo<EChartsOption>(() => {
    const quantities = new Map<string, number>()
    orders.forEach((order) => order.items.forEach((item) => {
      quantities.set(item.product_name, (quantities.get(item.product_name) ?? 0) + item.quantity)
    }))
    const popular = [...quantities.entries()].sort((left, right) => right[1] - left[1]).slice(0, 8).reverse()

    return {
      color: ['#5b8ff9'],
      tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
      grid: { top: 12, right: 30, bottom: 18, left: 110, containLabel: false },
      xAxis: { type: 'value', minInterval: 1, splitLine: { lineStyle: { color: '#f0f2f5' } }, axisLabel: { color: '#8994a4' } },
      yAxis: { type: 'category', inverse: true, data: popular.map(([name]) => name), axisTick: { show: false }, axisLine: { show: false }, axisLabel: { color: '#657286', width: 96, overflow: 'truncate' } },
      series: [{ type: 'bar', barMaxWidth: 22, data: popular.map(([, quantity]) => quantity), itemStyle: { borderRadius: [0, 6, 6, 0] }, label: { show: true, position: 'right', color: '#657286' } }],
      graphic: popular.length === 0 ? [{ type: 'text', left: 'center', top: 'middle', style: { text: '暂无订单', fill: '#a0a8b5', fontSize: 13 } }] : [],
    }
  }, [orders])

  return (
    <div className="page-stack">
      {error && <Alert type="warning" showIcon message={error} />}
      <Row gutter={[16, 16]}>
        <Col xs={12} xl={6}><Card className="stat-card dashboard-stat-card" loading={loading}><Statistic title="菜品总数" value={productStats?.total ?? '—'} suffix="款" /></Card></Col>
        <Col xs={12} xl={6}><Card className="stat-card dashboard-stat-card" loading={loading}><Statistic title="当前可点" value={productStats?.active ?? '—'} suffix="款" /></Card></Col>
        <Col xs={12} xl={6}><Card className="stat-card dashboard-stat-card" loading={loading}><Statistic title="订单总数" value={orderTotal ?? '—'} suffix="笔" /></Card></Col>
        <Col xs={12} xl={6}><Card className="stat-card dashboard-stat-card" loading={loading}><Statistic title="近 100 单实收" value={paidAmount} precision={2} prefix="¥" /></Card></Col>
      </Row>
      <Row gutter={[16, 16]}>
        <Col xs={24} xl={16}>
          <Card className="stat-card dashboard-chart-card" title="近 7 日趋势 · 最近 100 单" loading={loading}>
            <Chart option={trendOption} />
          </Card>
        </Col>
        <Col xs={24} xl={8}>
          <Card className="stat-card dashboard-chart-card" title="用餐方式 · 最近 100 单" loading={loading}>
            <Chart option={modeOption} />
          </Card>
        </Col>
      </Row>
      <Row gutter={[16, 16]}>
        <Col xs={24} xl={12}>
          <Card className="stat-card dashboard-chart-card" title="订单状态 · 最近 100 单" loading={loading}>
            <Chart option={statusOption} />
          </Card>
        </Col>
        <Col xs={24} xl={12}>
          <Card className="stat-card dashboard-chart-card" title="热销菜品 · 最近 100 单" loading={loading}>
            <Chart option={popularOption} />
          </Card>
        </Col>
      </Row>
    </div>
  )
}
