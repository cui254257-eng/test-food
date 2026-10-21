import { useEffect, useState } from 'react'
import { Alert, Button, Card, Form, Input, InputNumber, message, Space, Switch, Typography } from 'antd'
import { getRestaurantSettings, updateRestaurantSettings, type RestaurantSettings } from '@/api/restaurant'

export default function RestaurantPage() {
  const [form] = Form.useForm<RestaurantSettings>()
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    getRestaurantSettings()
      .then((settings) => form.setFieldsValue(settings))
      .catch((error: unknown) => message.error(error instanceof Error ? error.message : '门店配置加载失败'))
      .finally(() => setLoading(false))
  }, [form])

  async function save() {
    const values = await form.validateFields()
    setSaving(true)
    try {
      const settings = await updateRestaurantSettings(values)
      form.setFieldsValue(settings)
      message.success('门店配置已保存')
    } catch (error) {
      message.error(error instanceof Error ? error.message : '保存失败')
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="page-stack">
      <div className="page-heading">
        <div>
          <Typography.Title level={2}>门店与营业配置</Typography.Title>
          <Typography.Text type="secondary">配置营业时间、用餐方式和配送。</Typography.Text>
        </div>
        <Button type="primary" size="large" loading={saving} onClick={() => void save()}>保存配置</Button>
      </div>
      <Alert type="info" showIcon message="当前为单门店设置；后台接口尚未接入管理员登录，正式上线前需启用后台鉴权。" />
      <Card loading={loading} className="restaurant-settings-card">
        <Form form={form} layout="vertical" requiredMark={false} className="product-form">
          <Form.Item name="restaurant_name" label="门店名称" rules={[{ required: true, whitespace: true }, { max: 80 }]}>
            <Input maxLength={80} placeholder="例如：好味小馆" />
          </Form.Item>
          <Form.Item name="business_hours" label="营业时间" rules={[{ required: true, whitespace: true }, { max: 255 }]}>
            <Input maxLength={255} placeholder="例如：10:00-14:00，17:00-22:00" />
          </Form.Item>
          <Space size={22} wrap>
            <Form.Item name="is_open" label="当前接单" valuePropName="checked"><Switch checkedChildren="营业" unCheckedChildren="打烊" /></Form.Item>
            <Form.Item name="dine_in_enabled" label="堂食" valuePropName="checked"><Switch checkedChildren="开放" unCheckedChildren="关闭" /></Form.Item>
            <Form.Item name="takeaway_enabled" label="外带自取" valuePropName="checked"><Switch checkedChildren="开放" unCheckedChildren="关闭" /></Form.Item>
            <Form.Item name="delivery_enabled" label="外卖配送" valuePropName="checked"><Switch checkedChildren="开放" unCheckedChildren="关闭" /></Form.Item>
          </Space>
          <Form.Item name="store_address" label="门店地址" rules={[{ max: 255 }]}>
            <Input maxLength={255} placeholder="外卖配送需填写门店地址" />
          </Form.Item>
          <Form.Item name="contact_phone" label="联系电话" rules={[{ max: 20 }]}>
            <Input maxLength={20} placeholder="顾客联系门店的电话" />
          </Form.Item>
          <Space size={16} className="form-row" align="start">
            <Form.Item name="delivery_radius_km" label="配送范围（公里）" rules={[{ required: true }]}>
              <InputNumber min={0.1} max={100} precision={1} style={{ width: '100%' }} />
            </Form.Item>
            <Form.Item name="delivery_minimum_amount" label="外卖起送金额（元）" rules={[{ required: true }]}>
              <InputNumber min={0} precision={2} style={{ width: '100%' }} />
            </Form.Item>
            <Form.Item name="delivery_fee" label="配送费（元）" rules={[{ required: true }]}>
              <InputNumber min={0} precision={2} style={{ width: '100%' }} />
            </Form.Item>
          </Space>
        </Form>
      </Card>
    </div>
  )
}
