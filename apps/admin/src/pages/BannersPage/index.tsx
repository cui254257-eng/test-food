import { useEffect, useState, type ChangeEvent } from 'react'
import { DeleteOutlined, EditOutlined, PlusOutlined, UploadOutlined } from '@ant-design/icons'
import { Button, Card, Empty, Form, Input, InputNumber, message, Modal, Popconfirm, Spin, Typography } from 'antd'
import { assetUrl } from '../../api/client'
import { deleteBanner, getBanners, updateBanner, uploadBanner, type Banner } from '../../api/banners'

interface BannerFormValues {
  title?: string
  sort_order?: number | null
}

function readAsDataUrl(file: File) {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => typeof reader.result === 'string' ? resolve(reader.result) : reject(new Error('图片读取失败'))
    reader.onerror = () => reject(new Error('图片读取失败'))
    reader.readAsDataURL(file)
  })
}

export default function BannersPage() {
  const [banners, setBanners] = useState<Banner[]>([])
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [open, setOpen] = useState(false)
  const [editingBanner, setEditingBanner] = useState<Banner | null>(null)
  const [file, setFile] = useState<File | null>(null)
  const [preview, setPreview] = useState('')
  const [backgroundFile, setBackgroundFile] = useState<File | null>(null)
  const [backgroundPreview, setBackgroundPreview] = useState('')
  const [form] = Form.useForm<BannerFormValues>()

  async function loadBanners() {
    setLoading(true)
    try {
      setBanners(await getBanners())
    } catch (error) {
      message.error(error instanceof Error ? error.message : '门店活动图加载失败')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { void loadBanners() }, [])

  useEffect(() => () => {
    if (preview.startsWith('blob:')) URL.revokeObjectURL(preview)
    if (backgroundPreview.startsWith('blob:')) URL.revokeObjectURL(backgroundPreview)
  }, [preview, backgroundPreview])

  function openUpload() {
    form.resetFields()
    form.setFieldsValue({ sort_order: 0 })
    setEditingBanner(null)
    setFile(null)
    setPreview('')
    setBackgroundFile(null)
    setBackgroundPreview('')
    setOpen(true)
  }

  function openEdit(banner: Banner) {
    form.resetFields()
    form.setFieldsValue({ title: banner.title, sort_order: banner.sort_order })
    setEditingBanner(banner)
    setFile(null)
    setPreview(assetUrl(banner.image_url))
    setBackgroundFile(null)
    setBackgroundPreview(banner.background_image_url ? assetUrl(banner.background_image_url) : '')
    setOpen(true)
  }

  function closeModal() {
    setOpen(false)
    setEditingBanner(null)
    setFile(null)
    setPreview('')
    setBackgroundFile(null)
    setBackgroundPreview('')
    form.resetFields()
  }

  function chooseImage(event: ChangeEvent<HTMLInputElement>, target: 'foreground' | 'background') {
    const nextFile = event.target.files?.[0]
    if (!nextFile) return
    if (!['image/jpeg', 'image/png', 'image/webp', 'image/gif'].includes(nextFile.type)) {
      message.error('请选择 JPG、PNG、WEBP 或 GIF 图片')
      event.target.value = ''
      return
    }
    if (nextFile.size > 5 * 1024 * 1024) {
      message.error('图片大小不能超过 5 MB')
      event.target.value = ''
      return
    }
    if (target === 'foreground') {
      setFile(nextFile)
      setPreview(URL.createObjectURL(nextFile))
    } else {
      setBackgroundFile(nextFile)
      setBackgroundPreview(URL.createObjectURL(nextFile))
    }
    event.target.value = ''
  }

  async function saveBanner() {
    const values = await form.validateFields()
    if (!file && !editingBanner) {
      message.error('请选择一张图片')
      return
    }
    setSaving(true)
    try {
      const imageData = file ? await readAsDataUrl(file) : undefined
      const backgroundImageData = backgroundFile ? await readAsDataUrl(backgroundFile) : undefined
      const input = {
        title: values.title?.trim() ?? '',
        sort_order: values.sort_order ?? 0,
        ...(imageData ? { image_data: imageData } : {}),
        ...(backgroundImageData ? { background_image_data: backgroundImageData } : {}),
      }
      if (editingBanner) {
        await updateBanner(editingBanner.id, input)
        message.success('门店活动图已更新')
      } else {
        await uploadBanner({ ...input, image_data: imageData! })
        message.success('门店活动图已上传')
      }
      closeModal()
      await loadBanners()
    } catch (error) {
      message.error(error instanceof Error ? error.message : '上传失败')
    } finally {
      setSaving(false)
    }
  }

  async function removeBanner(id: number) {
    try {
      await deleteBanner(id)
      message.success('门店活动图已删除')
      await loadBanners()
    } catch (error) {
      message.error(error instanceof Error ? error.message : '删除失败')
    }
  }

  return (
    <div className="page-stack">
      <div className="page-heading">
        <div><Typography.Title level={2}>门店活动图</Typography.Title></div>
        <Button type="primary" size="large" icon={<PlusOutlined />} onClick={openUpload}>上传活动图片</Button>
      </div>

      {loading && banners.length === 0 ? (
        <div className="banner-empty"><Spin /></div>
      ) : banners.length === 0 ? (
        <Card className="banner-empty"><Empty description="还没有门店活动图" /></Card>
      ) : (
        <div className="banner-grid">
          {banners.map((banner) => (
            <Card key={banner.id} className="banner-card" loading={loading} cover={(
              <div className="banner-card-cover">
                <img className="banner-card-background" src={assetUrl(banner.background_image_url || banner.image_url)} alt="" />
                <img className="banner-card-foreground" src={assetUrl(banner.image_url)} alt={banner.title || '门店活动图'} />
              </div>
            )}>
              <div className="banner-card-footer">
                <div><strong>{banner.title || '未命名'}</strong><small>排序 {banner.sort_order}</small></div>
                <div className="banner-card-actions">
                  <Button type="text" icon={<EditOutlined />} aria-label="编辑轮播图" onClick={() => openEdit(banner)} />
                  <Popconfirm title="删除这张轮播图？" onConfirm={() => void removeBanner(banner.id)}>
                    <Button type="text" danger icon={<DeleteOutlined />} aria-label="删除轮播图" />
                  </Popconfirm>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}

      <Modal
        title={editingBanner ? '编辑门店活动图' : '上传门店活动图'}
        open={open}
        onCancel={closeModal}
        onOk={() => void saveBanner()}
        confirmLoading={saving}
        okText={editingBanner ? '保存修改' : '上传'}
        cancelText="取消"
        destroyOnHidden
      >
        <Form form={form} layout="vertical" requiredMark={false} initialValues={{ sort_order: 0 }}>
          <Form.Item name="title" label="标题" rules={[{ max: 80, message: '最多 80 个字' }]}>
            <Input placeholder="可不填" maxLength={80} />
          </Form.Item>
          <Form.Item name="sort_order" label="排序值" rules={[{ required: true, message: '请输入排序值' }]}>
            <InputNumber min={0} max={2147483647} precision={0} style={{ width: '100%' }} />
          </Form.Item>
          <Typography.Text type="secondary" className="banner-upload-hint">数值越小越靠前。</Typography.Text>
          <Typography.Text strong>前景轮播图</Typography.Text>
          <div className="banner-upload-field">
            <input type="file" accept="image/jpeg,image/png,image/webp,image/gif" onChange={(event) => chooseImage(event, 'foreground')} />
            {preview ? <img src={preview} alt="前景图预览" /> : <div className="banner-upload-placeholder"><UploadOutlined /><span>选择前景图片</span></div>}
          </div>
          <Typography.Text type="secondary" className="banner-upload-hint">
            {editingBanner ? '可选：重新选择图片替换当前前景图；不选择则保留原图。' : '必填，JPG、PNG、WEBP、GIF，最大 5 MB'}
          </Typography.Text>
          <Typography.Text strong className="banner-background-label">背景轮播图（可选）</Typography.Text>
          <div className="banner-upload-field">
            <input type="file" accept="image/jpeg,image/png,image/webp,image/gif" onChange={(event) => chooseImage(event, 'background')} />
            {backgroundPreview ? <img src={backgroundPreview} alt="背景图预览" /> : <div className="banner-upload-placeholder"><UploadOutlined /><span>选择背景图片</span></div>}
          </div>
          <Typography.Text type="secondary" className="banner-upload-hint">
            {editingBanner ? '不选择则保留原背景图；没有背景图时会用前景图铺底。' : '用于前景图下层的满幅背景，JPG、PNG、WEBP、GIF，最大 5 MB。'}
          </Typography.Text>
        </Form>
      </Modal>
    </div>
  )
}
