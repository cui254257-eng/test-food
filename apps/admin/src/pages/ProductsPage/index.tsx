import { useEffect, useState, type ChangeEvent } from 'react'
import type { Product, ProductInput } from '@shop/contracts'
import { Button, Form, Input, InputNumber, message, Modal, Popconfirm, Select, Space, Switch, Table, Tag, Typography } from 'antd'
import type { TableProps } from 'antd'
import { assetUrl } from '@/api/client'
import { createProduct, deleteProduct, getProducts, updateProduct } from '@/api/products'

type ProductFormValues = ProductInput

const currency = new Intl.NumberFormat('zh-CN', { style: 'currency', currency: 'CNY' })

export default function ProductsPage() {
  const [products, setProducts] = useState<Product[]>([])
  const [query, setQuery] = useState('')
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [open, setOpen] = useState(false)
  const [editing, setEditing] = useState<Product | null>(null)
  const [imageFile, setImageFile] = useState<File | null>(null)
  const [imagePreview, setImagePreview] = useState('')
  const [form] = Form.useForm<ProductFormValues>()

  async function loadProducts(search = query) {
    setLoading(true)
    try {
      setProducts(await getProducts(search))
    } catch (error) {
      message.error(error instanceof Error ? error.message : '菜品加载失败')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { void loadProducts('') }, [])

  useEffect(() => {
    if (!imagePreview.startsWith('blob:')) return
    return () => URL.revokeObjectURL(imagePreview)
  }, [imagePreview])

  function openCreate() {
    setEditing(null)
    setImageFile(null)
    setImagePreview('')
    form.resetFields()
    form.setFieldsValue({ name: '', category: '主食盖饭', description: '', specifications: [], price: 0, stock: 0, status: 'active' })
    setOpen(true)
  }

  function openEdit(product: Product) {
    setEditing(product)
    setImageFile(null)
    setImagePreview(product.image_url ? assetUrl(product.image_url) : '')
    form.setFieldsValue({
      name: product.name,
      category: product.category,
      description: product.description,
      specifications: product.specifications ?? [],
      price: product.price,
      stock: product.stock,
      status: product.status,
    })
    setOpen(true)
  }

  function closeModal() {
    setOpen(false)
    setEditing(null)
    setImageFile(null)
    setImagePreview('')
    form.resetFields()
  }

  function chooseImage(event: ChangeEvent<HTMLInputElement>) {
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
    setImageFile(nextFile)
    setImagePreview(URL.createObjectURL(nextFile))
  }

  function readAsDataUrl(file: File) {
    return new Promise<string>((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => typeof reader.result === 'string' ? resolve(reader.result) : reject(new Error('图片读取失败'))
      reader.onerror = () => reject(new Error('图片读取失败'))
      reader.readAsDataURL(file)
    })
  }

  async function saveProduct() {
    const values = await form.validateFields()
    setSaving(true)
    try {
      const input: ProductInput = {
        ...values,
        ...(imageFile ? { image_data: await readAsDataUrl(imageFile) } : {}),
      }
      if (editing) await updateProduct(editing.id, input)
      else await createProduct(input)
      message.success(editing ? '菜品已更新' : '菜品已添加')
      closeModal()
      await loadProducts()
    } catch (error) {
      message.error(error instanceof Error ? error.message : '保存失败')
    } finally {
      setSaving(false)
    }
  }

  async function removeProduct(id: number) {
    try {
      await deleteProduct(id)
      message.success('菜品已删除')
      await loadProducts()
    } catch (error) {
      message.error(error instanceof Error ? error.message : '删除失败')
    }
  }

  const columns: TableProps<Product>['columns'] = [
    { title: '菜品', dataIndex: 'name', key: 'name', render: (name: string, product) => <div className="product-cell">{product.image_url ? <img className="product-avatar-image" src={assetUrl(product.image_url)} alt={name} /> : <span className="product-avatar">{name.slice(0, 1)}</span>}<span><strong>{name}</strong></span></div> },
    { title: '分类', dataIndex: 'category', key: 'category', width: 110 },
    { title: '菜品描述', dataIndex: 'description', key: 'description', ellipsis: true, render: (text: string) => text || '—' },
    { title: '售价（元/份）', dataIndex: 'price', key: 'price', width: 140, render: (price: number) => <strong>{currency.format(price)}</strong> },
    { title: '可售份数', dataIndex: 'stock', key: 'stock', width: 100 },
    { title: '状态', dataIndex: 'status', key: 'status', width: 110, render: (status: Product['status']) => <Tag color={status === 'active' ? 'green' : 'default'}>{status === 'active' ? '可点餐' : '暂停售卖'}</Tag> },
    { title: '操作', key: 'action', width: 150, render: (_, product) => <Space><Button type="link" onClick={() => openEdit(product)}>编辑</Button><Popconfirm title="确定删除这道菜品？" onConfirm={() => void removeProduct(product.id)}><Button type="link" danger>删除</Button></Popconfirm></Space> },
  ]

  return (
    <div className="page-stack">
      <div className="page-heading">
        <div>
          <Typography.Title level={2}>菜品菜单</Typography.Title>
        </div>
        <Button type="primary" size="large" onClick={openCreate}>新增菜品</Button>
      </div>
      <div className="table-toolbar">
        <Input.Search
          allowClear
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          onSearch={(value) => void loadProducts(value)}
          placeholder="搜索菜品"
          style={{ width: 300 }}
        />
        <Typography.Text type="secondary">{products.length} 款</Typography.Text>
      </div>
      <Table<Product>
        rowKey="id"
        columns={columns}
        dataSource={products}
        loading={loading}
        pagination={{ pageSize: 8, showSizeChanger: false }}
        className="products-table"
      />
      <Modal
        title={editing ? '编辑菜品' : '新增菜品'}
        open={open}
        onCancel={closeModal}
        onOk={() => void saveProduct()}
        confirmLoading={saving}
        okText="保存"
        cancelText="取消"
        destroyOnHidden
      >
        <Form form={form} layout="vertical" requiredMark={false} className="product-form">
          <Form.Item name="name" label="菜品名称" rules={[{ required: true, whitespace: true, message: '请输入菜品名称' }, { max: 80, message: '名称不能超过 80 个字' }]}>
            <Input placeholder="例如：招牌红烧牛肉饭" maxLength={80} />
          </Form.Item>
          <Form.Item name="category" label="菜单分类" rules={[{ required: true, message: '请选择菜单分类' }]}>
            <Select options={['主食盖饭', '面食', '小吃', '饮品', '甜品', '套餐'].map((category) => ({ value: category, label: category }))} />
          </Form.Item>
          <Form.Item name="description" label="描述" rules={[{ max: 500, message: '最多 500 字' }]}>
            <Input.TextArea placeholder="例如：慢炖牛腩，配米饭和时蔬" rows={2} maxLength={120} showCount />
          </Form.Item>
          <Form.List name="specifications">
            {(groups, { add: addGroup, remove: removeGroup }) => (
              <div className="menu-specifications">
                <div className="specification-heading">
                  <div><strong>菜品规格</strong><small>例如：份量、辣度；选项可设置加价</small></div>
                  <Button onClick={() => addGroup({ name: '', required: false, options: [] })}>添加规格组</Button>
                </div>
                {groups.map((group) => (
                  <div className="specification-group" key={group.key}>
                    <Space align="start" className="form-row">
                      <Form.Item name={[group.name, 'name']} label="规格名称" rules={[{ required: true, whitespace: true, message: '请输入规格名称' }, { max: 30 }]}>
                        <Input placeholder="份量 / 辣度" maxLength={30} />
                      </Form.Item>
                      <Form.Item name={[group.name, 'required']} label="顾客必选" valuePropName="checked">
                        <Switch />
                      </Form.Item>
                      <Button danger type="text" onClick={() => removeGroup(group.name)}>删除规格组</Button>
                    </Space>
                    <Form.List
                      name={[group.name, 'options']}
                      rules={[{ validator: async (_, value: unknown[]) => value?.length ? Promise.resolve() : Promise.reject(new Error('每组规格至少添加一个选项')) }]}
                    >
                      {(options, { add: addOption, remove: removeOption }, { errors }) => (
                        <>
                          {options.map((option) => (
                            <Space align="start" key={option.key} className="form-row">
                              <Form.Item name={[option.name, 'name']} label="选项" rules={[{ required: true, whitespace: true, message: '请输入选项名称' }, { max: 50 }]}>
                                <Input placeholder="例如：大份" maxLength={50} />
                              </Form.Item>
                              <Form.Item name={[option.name, 'price_delta']} label="加价（元）" rules={[{ required: true }]}>
                                <InputNumber min={0} precision={2} style={{ width: '100%' }} />
                              </Form.Item>
                              <Button danger type="text" onClick={() => removeOption(option.name)}>移除</Button>
                            </Space>
                          ))}
                          <Button type="dashed" block onClick={() => addOption({ name: '', price_delta: 0 })}>添加规格选项</Button>
                          <Form.ErrorList errors={errors} />
                        </>
                      )}
                    </Form.List>
                  </div>
                ))}
              </div>
            )}
          </Form.List>
          <Form.Item label="菜品图片">
            <div className="product-image-upload">
              <input type="file" accept="image/jpeg,image/png,image/webp,image/gif" onChange={chooseImage} />
              {imagePreview ? <img src={imagePreview} alt="菜品图片预览" /> : <span>点击选择菜品图片</span>}
            </div>
            <Typography.Text type="secondary" className="banner-upload-hint">
              {editing ? 'JPG、PNG、WEBP、GIF，最大 5 MB；不选新图片则保留原图。' : '可选，JPG、PNG、WEBP、GIF，最大 5 MB。菜品图片将以 Base64 保存。'}
            </Typography.Text>
          </Form.Item>
          <Space size={16} className="form-row" align="start">
            <Form.Item name="price" label="售价（元/份）" rules={[{ required: true, message: '请输入菜品售价' }]}>
              <InputNumber min={0} precision={2} step={0.01} style={{ width: '100%' }} />
            </Form.Item>
            <Form.Item name="stock" label="今日可售份数" rules={[{ required: true, message: '请输入今日可售份数' }]}>
              <InputNumber min={0} precision={0} style={{ width: '100%' }} />
            </Form.Item>
          </Space>
          <Form.Item name="status" label="状态" rules={[{ required: true }]}>
            <Select options={[{ value: 'active', label: '可点餐' }, { value: 'inactive', label: '暂停售卖' }]} />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  )
}
