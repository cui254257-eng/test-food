import { useState } from "react";
import { LockOutlined, ShopOutlined, UserOutlined } from "@ant-design/icons";
import { Button, Form, Input, Typography, message } from "antd";
import { Navigate, useLocation, useNavigate } from "react-router-dom";
import { getAdminToken, loginAdmin, saveAdminToken } from "../../api/adminAuth";

interface LoginValues {
  username: string;
  password: string;
}

export default function LoginPage() {
  const [loading, setLoading] = useState(false);
  const location = useLocation();
  const navigate = useNavigate();
  const token = getAdminToken();
  const redirectTo =
    (location.state as { from?: { pathname?: string } } | null)?.from
      ?.pathname ?? "/dashboard";

  if (token) return <Navigate to={redirectTo} replace />;

  async function submit(values: LoginValues) {
    setLoading(true);
    try {
      const result = await loginAdmin(values);
      saveAdminToken(result.token);
      message.success("登录成功");
      navigate(redirectTo, { replace: true });
    } catch (error) {
      message.error(
        error instanceof Error ? error.message : "登录失败，请稍后重试",
      );
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="login-page">
      <section className="login-showcase">
        <div className="login-showcase-copy">
          <span className="login-eyebrow">STORE MANAGEMENT</span>
          <Typography.Title>
            把好味道，
            <br />
            好好经营。
          </Typography.Title>
          <Typography.Paragraph>
            菜单、订单与门店运营，在这里一站打理。
          </Typography.Paragraph>
        </div>
        <div className="login-showcase-foot">好味小馆 · 管理后台</div>
      </section>
      <section className="login-form-wrap">
        <div className="login-form-card">
          <div className="login-mobile-brand">
            <ShopOutlined /> 好味小馆
          </div>
          <Typography.Title level={2}>欢迎回来</Typography.Title>
          <Typography.Paragraph type="secondary">
            登录以继续管理你的门店
          </Typography.Paragraph>
          <Form<LoginValues>
            layout="vertical"
            size="large"
            onFinish={submit}
            requiredMark={false}
          >
            <Form.Item
              label="管理员账号"
              name="username"
              rules={[{ required: true, message: "请输入管理员账号" }]}
            >
              <Input
                prefix={<UserOutlined />}
                autoComplete="username"
                placeholder="请输入管理员账号"
              />
            </Form.Item>
            <Form.Item
              label="登录密码"
              name="password"
              rules={[{ required: true, message: "请输入登录密码" }]}
            >
              <Input.Password
                prefix={<LockOutlined />}
                autoComplete="current-password"
                placeholder="请输入登录密码"
              />
            </Form.Item>
            <Button
              type="primary"
              htmlType="submit"
              block
              loading={loading}
              className="login-submit"
            >
              登录管理后台
            </Button>
          </Form>
        </div>
      </section>
    </main>
  );
}
