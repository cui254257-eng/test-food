import { useEffect, useState } from "react";
import {
  Alert,
  Button,
  Card,
  Form,
  InputNumber,
  message,
  Space,
  Typography,
} from "antd";
import {
  getMembershipConfig,
  updateMembershipConfig,
  type MembershipConfig,
} from "@/api/membership";

interface MembershipFormValues {
  spend_step_yuan: number;
  points_per_step: number;
  tiers: Array<{ required_points: number; discount_percent: number }>;
}

const defaultTiers = [
  { level: 1, required_points: 0, discount_rate: 1 },
  { level: 2, required_points: 100, discount_rate: 0.98 },
  { level: 3, required_points: 300, discount_rate: 0.95 },
  { level: 4, required_points: 1000, discount_rate: 0.93 },
  { level: 5, required_points: 3000, discount_rate: 0.9 },
];

function toFormValues(config: MembershipConfig): MembershipFormValues {
  return {
    spend_step_yuan: config.spend_step_cents / 100,
    points_per_step: config.points_per_step,
    tiers: config.tiers.map((tier) => ({
      required_points: tier.required_points,
      discount_percent: tier.discount_rate * 100,
    })),
  };
}

function toConfig(values: MembershipFormValues): MembershipConfig {
  const formTiers: MembershipFormValues["tiers"] = values.tiers?.length
    ? values.tiers
    : defaultTiers.map(({ required_points, discount_rate }) => ({
        required_points,
        discount_percent: discount_rate * 100,
      }));
  const tiers = formTiers.map((tier, index) => ({
    level: index + 1,
    required_points: tier.required_points,
    discount_rate: tier.discount_percent / 100,
  }));
  return {
    spend_step_cents: Math.round(values.spend_step_yuan * 100),
    points_per_step: values.points_per_step,
    tiers,
  };
}

export default function MembershipPage() {
  const [form] = Form.useForm<MembershipFormValues>();
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    getMembershipConfig()
      .then((config) => form.setFieldsValue(toFormValues(config)))
      .catch((reason: unknown) =>
        setError(reason instanceof Error ? reason.message : "会员规则加载失败"),
      )
      .finally(() => setLoading(false));
  }, [form]);

  async function save() {
    const values = await form.validateFields();
    setSaving(true);
    setError("");
    try {
      const saved = await updateMembershipConfig(toConfig(values));
      form.setFieldsValue(toFormValues(saved));
      message.success("会员规则已保存");
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : "保存失败");
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="page-stack">
      <div className="page-heading">
        <Typography.Title level={2}>会员规则</Typography.Title>
        <Button type="primary" onClick={() => void save()} loading={saving}>
          保存
        </Button>
      </div>
      {error && <Alert type="error" showIcon message={error} />}
      <Card className="stat-card" loading={loading}>
        <Form form={form} layout="vertical" requiredMark={false}>
          <Typography.Title level={5}>积分获取</Typography.Title>
          <Space align="start" size={18} className="form-row">
            <Form.Item
              name="spend_step_yuan"
              label="每消费（元）"
              rules={[
                { required: true },
                { type: "number", min: 0.01, max: 100000 },
              ]}
            >
              <InputNumber
                min={0.01}
                precision={2}
                addonAfter="元"
                style={{ width: "100%" }}
              />
            </Form.Item>
            <Form.Item
              name="points_per_step"
              label="获得积分"
              rules={[
                { required: true },
                { type: "number", min: 1, max: 1000000 },
              ]}
            >
              <InputNumber
                min={1}
                precision={0}
                addonAfter="分"
                style={{ width: "100%" }}
              />
            </Form.Item>
          </Space>
          <Typography.Title level={5} style={{ marginTop: 12 }}>
            等级与折扣
          </Typography.Title>
          <div className="membership-tier-grid">
            {defaultTiers.map((tier, index) => (
              <div className="membership-tier-row" key={tier.level}>
                <strong>V{tier.level}</strong>
                <Form.Item
                  name={["tiers", index, "required_points"]}
                  label="升级积分"
                  rules={[
                    { required: true },
                    { type: "number", min: 0, max: 100000000 },
                  ]}
                >
                  <InputNumber
                    min={0}
                    precision={0}
                    addonAfter="分"
                    style={{ width: "100%" }}
                  />
                </Form.Item>
                <Form.Item
                  name={["tiers", index, "discount_percent"]}
                  label="折扣"
                  rules={[
                    { required: true },
                    { type: "number", min: 50, max: 100 },
                  ]}
                >
                  <InputNumber
                    min={50}
                    max={100}
                    precision={1}
                    addonAfter="%"
                    style={{ width: "100%" }}
                  />
                </Form.Item>
              </div>
            ))}
          </div>
          <Typography.Paragraph type="secondary" style={{ marginBottom: 0 }}>
            新会员为
            V1，不打折；积分按实付商品金额累计，配送费不计积分。积分零头不超过
            0.06 分舍弃，超过 0.06 分进一分；折扣只应用于已开启会员折扣的菜品。
          </Typography.Paragraph>
        </Form>
      </Card>
    </div>
  );
}
