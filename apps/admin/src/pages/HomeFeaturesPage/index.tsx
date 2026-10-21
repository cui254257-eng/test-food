import { useEffect, useState } from 'react'
import { DeleteOutlined, EditOutlined, PlusOutlined } from '@ant-design/icons'
import { Button, Form, Input, InputNumber, message, Modal, Popconfirm, Select, Space, Switch, Table, Tag, Typography } from 'antd'
import type { TableProps } from 'antd'
import { assetUrl } from '@/api/client'
import {
  createHomeFeature,
  deleteHomeFeature,
  getHomeFeatures,
  updateHomeFeature,
  type HomeFeature,
  type HomeFeatureInput,
} from '@/api/homeFeatures'

const iconOptions = [
  { value: 'all.svg', label: '全部菜品' },
  { value: 'cart.svg', label: '点餐篮' },
  { value: 'coffee.svg', label: '现制饮品' },
  { value: 'home.svg', label: '招牌盖饭' },
  { value: 'hot.svg', label: '人气热卖' },
  { value: 'new.svg', label: '今日上新' },
  { value: 'price.svg', label: '优惠套餐' },
  { value: 'snacks.svg', label: '面食小吃' },
]

export default function HomeFeaturesPage() {
  const [features, setFeatures] = useState<HomeFeature[]>([])
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [open, setOpen] = useState(false)
  const [editing, setEditing] = useState<HomeFeature | null>(null)
  const [form] = Form.useForm<HomeFeatureInput>()

  async function loadFeatures() {
    setLoading(true)
    try {
      setFeatures(await getHomeFeatures())
    } catch (error) {
      message.error(error instanceof Error ? error.message : '点餐分类加载失败')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { void loadFeatures() }, [])

  function openCreate() {
    setEditing(null)
    form.resetFields()
      form.setFieldsValue({ label: '', icon: 'all.svg', target_path: '/pages/products/index', sort_order: 0, is_active: true })
    setOpen(true)
  }

  function openEdit(feature: HomeFeature) {
    setEditing(feature)
    form.setFieldsValue({
      label: feature.label,
      icon: feature.icon,
      target_path: feature.target_path,
      sort_order: feature.sort_order,
      is_active: feature.is_active,
    })
    setOpen(true)
  }

  function closeModal() {
    setOpen(false)
    setEditing(null)
    form.resetFields()
  }

  async function saveFeature() {
    const values = await form.validateFields()
    setSaving(true)
    try {
      if (editing) {
        await updateHomeFeature(editing.id, values)
        message.success('点餐分类已更新')
      } else {
        await createHomeFeature(values)
        message.success('点餐分类已新增')
      }
      closeModal()
      await loadFeatures()
    } catch (error) {
      message.error(error instanceof Error ? error.message : '保存失败')
    } finally {
      setSaving(false)
    }
  }

  async function removeFeature(id: number) {
    try {
      await deleteHomeFeature(id)
      message.success('点餐分类已删除')
      await loadFeatures()
    } catch (error) {
      message.error(error instanceof Error ? error.message : '删除失败')
    }
  }

  const columns: TableProps<HomeFeature>['columns'] = [
    { title: '模块', key: 'module', render: (_, feature) => <div className="feature-admin-cell"><img src={assetUrl(`/assets/icons/${feature.icon}`)} alt="" /><span><strong>{feature.label}</strong></span></div> },
    { title: '跳转地址', dataIndex: 'target_path', key: 'target_path', ellipsis: true, render: (path: string) => <code>{path}</code> },
    { title: '排序', dataIndex: 'sort_order', key: 'sort_order', width: 90 },
    { title: '状态', dataIndex: 'is_active', key: 'is_active', width: 90, render: (active: boolean) => <Tag color={active ? 'green' : 'default'}>{active ? '展示中' : '已隐藏'}</Tag> },
    { title: '操作', key: 'action', width: 150, render: (_, feature) => <Space><Button type="link" icon={<EditOutlined />} onClick={() => openEdit(feature)}>编辑</Button><Popconfirm title="删除这个点餐分类入口？" onConfirm={() => void removeFeature(feature.id)}><Button type="link" danger icon={<DeleteOutlined />}>删除</Button></Popconfirm></Space> },
  ]

  return (
    <div className="page-stack">
      <div className="page-heading">
        <div>
          <Typography.Title level={2}>首页点餐分类</Typography.Title>
          <Typography.Text type="secondary">配置首页菜品分类入口和排序。</Typography.Text>
        </div>
        <Button type="primary" size="large" icon={<PlusOutlined />} onClick={openCreate}>新增分类入口</Button>
      </div>
      <Table<HomeFeature>
        rowKey="id"
        columns={columns}
        dataSource={features}
        loading={loading}
        pagination={{ pageSize: 10, showSizeChanger: false }}
        className="products-table"
      />
      <Modal
        title={editing ? '编辑点餐分类' : '新增点餐分类'}
        open={open}
        onCancel={closeModal}
        onOk={() => void saveFeature()}
        confirmLoading={saving}
        okText="保存"
        cancelText="取消"
        destroyOnHidden
      >
        <Form form={form} layout="vertical" requiredMark={false} className="product-form">
          <Form.Item name="label" label="显示名称" rules={[{ required: true, whitespace: true, message: '请输入模块名称' }, { max: 40, message: '最多 40 个字' }]}>
          <Input maxLength={40} placeholder="例如：招牌盖饭" />
          </Form.Item>
          <Form.Item name="icon" label="图标" rules={[{ required: true, message: '请选择图标' }]}>
            <Select options={iconOptions.map((option) => ({ ...option, label: <span className="feature-icon-option"><img src={assetUrl(`/assets/icons/${option.value}`)} alt="" />{option.label}</span> }))} />
          </Form.Item>
          <Form.Item name="target_path" label="跳转地址" rules={[{ required: true, whitespace: true, message: '请输入小程序页面路径' }, { max: 255, message: '地址不能超过 255 个字符' }, { pattern: /^\/pages\/[^\s]+$/, message: '请输入 /pages/ 开头的小程序页面地址' }]}>
          <Input placeholder="/pages/products/index?q=牛肉饭" />
          </Form.Item>
          <Typography.Text type="secondary" className="banner-upload-hint">填写已注册的小程序页面路径；菜品详情需带菜品参数。</Typography.Text>
          <Space size={16} className="form-row" align="start">
            <Form.Item name="sort_order" label="排序值" rules={[{ required: true, message: '请输入排序值' }]}>
              <InputNumber min={0} precision={0} style={{ width: '100%' }} />
            </Form.Item>
            <Form.Item name="is_active" label="在首页展示" valuePropName="checked">
              <Switch checkedChildren="展示" unCheckedChildren="隐藏" />
            </Form.Item>
          </Space>
        </Form>
      </Modal>
    </div>
  )
}
