import { useEffect, useState } from 'react'
import { DeleteOutlined, EditOutlined, PlusOutlined } from '@ant-design/icons'
import {
  Alert,
  Button,
  Card,
  Empty,
  Form,
  Input,
  InputNumber,
  message,
  Modal,
  Popconfirm,
  Space,
  Switch,
  Table,
  Tag,
  Typography,
} from 'antd'
import type { ColumnsType } from 'antd/es/table'
import {
  createAnnouncement,
  deleteAnnouncement,
  getAnnouncements,
  updateAnnouncement,
  type Announcement,
  type AnnouncementInput,
} from '@/api/announcements'

export default function AnnouncementPage() {
  const [announcements, setAnnouncements] = useState<Announcement[]>([])
  const [form] = Form.useForm<AnnouncementInput>()
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [open, setOpen] = useState(false)
  const [editingAnnouncement, setEditingAnnouncement] = useState<Announcement | null>(null)

  async function loadAnnouncements() {
    setLoading(true)
    try {
      setAnnouncements(await getAnnouncements())
    } catch (error) {
      message.error(error instanceof Error ? error.message : '公告加载失败')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { void loadAnnouncements() }, [])

  function openCreate() {
    setEditingAnnouncement(null)
    form.resetFields()
    form.setFieldsValue({ is_active: true, sort_order: 0 })
    setOpen(true)
  }

  function openEdit(announcement: Announcement) {
    setEditingAnnouncement(announcement)
    form.setFieldsValue({
      content: announcement.content,
      is_active: announcement.is_active,
      sort_order: announcement.sort_order,
    })
    setOpen(true)
  }

  function closeModal() {
    setOpen(false)
    setEditingAnnouncement(null)
    form.resetFields()
  }

  async function save() {
    const values = await form.validateFields()
    const input = { ...values, content: values.content.trim() }
    setSaving(true)
    try {
      if (editingAnnouncement) {
        await updateAnnouncement(editingAnnouncement.id, input)
        message.success('公告已更新')
      } else {
        await createAnnouncement(input)
        message.success('公告已添加')
      }
      closeModal()
      await loadAnnouncements()
    } catch (error) {
      message.error(error instanceof Error ? error.message : '公告保存失败')
    } finally {
      setSaving(false)
    }
  }

  async function remove(id: number) {
    try {
      await deleteAnnouncement(id)
      message.success('公告已删除')
      await loadAnnouncements()
    } catch (error) {
      message.error(error instanceof Error ? error.message : '公告删除失败')
    }
  }

  const columns: ColumnsType<Announcement> = [
    {
      title: '公告内容',
      dataIndex: 'content',
      key: 'content',
      render: (content: string) => (
        <Typography.Paragraph ellipsis={{ rows: 2, tooltip: true }} style={{ margin: 0 }}>
          {content}
        </Typography.Paragraph>
      ),
    },
    { title: '排序', dataIndex: 'sort_order', key: 'sort_order', width: 90, sorter: (a, b) => a.sort_order - b.sort_order },
    {
      title: '状态',
      dataIndex: 'is_active',
      key: 'is_active',
      width: 100,
      render: (active: boolean) => active ? <Tag color="green">显示中</Tag> : <Tag>已关闭</Tag>,
    },
    {
      title: '操作',
      key: 'actions',
      width: 120,
      render: (_, announcement) => (
        <Space>
          <Button type="text" icon={<EditOutlined />} aria-label="编辑公告" onClick={() => openEdit(announcement)} />
          <Popconfirm title="删除这条公告？" onConfirm={() => void remove(announcement.id)}>
            <Button type="text" danger icon={<DeleteOutlined />} aria-label="删除公告" />
          </Popconfirm>
        </Space>
      ),
    },
  ]

  return (
    <div className="page-stack">
      <div className="page-heading">
        <div>
          <Typography.Title level={2}>首页公告</Typography.Title>
          <Typography.Text type="secondary">小程序首页按排序展示启用的公告，可左右滑动翻页。</Typography.Text>
        </div>
        <Button type="primary" size="large" icon={<PlusOutlined />} onClick={openCreate}>新增公告</Button>
      </div>
      <Alert type="info" showIcon message="可以添加多条公告；排序值越小越靠前，关闭的公告不会显示在小程序中。" />
      <Card className="restaurant-settings-card">
        <Table<Announcement>
          rowKey="id"
          columns={columns}
          dataSource={announcements}
          loading={loading}
          locale={{ emptyText: <Empty description="还没有公告" /> }}
          pagination={{ pageSize: 10, showSizeChanger: false, hideOnSinglePage: true }}
        />
      </Card>
      <Modal
        title={editingAnnouncement ? '编辑公告' : '新增公告'}
        open={open}
        onCancel={closeModal}
        onOk={() => void save()}
        confirmLoading={saving}
        okText={editingAnnouncement ? '保存修改' : '添加'}
        cancelText="取消"
        destroyOnHidden
      >
        <Form form={form} layout="vertical" requiredMark={false} className="product-form">
          <Form.Item name="content" label="公告内容" rules={[{ required: true, whitespace: true, message: '请填写公告内容' }, { max: 255, message: '最多 255 个字' }]}>
            <Input.TextArea rows={3} maxLength={255} showCount placeholder="例如：今日堂食营业至 21:00" />
          </Form.Item>
          <Form.Item name="sort_order" label="排序值" rules={[{ required: true, message: '请输入排序值' }]}>
            <InputNumber min={0} max={2147483647} precision={0} style={{ width: '100%' }} />
          </Form.Item>
          <Form.Item name="is_active" label="显示公告" valuePropName="checked">
            <Switch checkedChildren="显示" unCheckedChildren="关闭" />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  )
}
