import React, { useEffect, useState } from "react";
import type { FormProps } from "antd";
import { Button, Form, Input, message, Card } from "antd";
import axios from "axios";
import restful_api from "./utils/restful_api.ts";
import { useNavigate } from "react-router-dom";
import JSEncrypt from "jsencrypt";

type FieldType = {
  username?: string;
  password?: string;
  captcha?: string;
};

const App: React.FC = () => {
  const navigate = useNavigate();
  const [captchaImg, setCaptchaImg] = useState<string>("");
  const [captchaUuid, setCaptchaUuid] = useState<string>("");
  const [publicKey, setPublicKey] = useState<string>("");
  const [logo, setLogo] = useState("/android-chrome-512x512.png");

  // 获取验证码
  const getCaptcha = async () => {
    try {
      const rsp = await restful_api.get("/api/captcha");
      const { uuid, base64_captcha, public_key } = rsp.data;
      setCaptchaUuid(uuid);
      setCaptchaImg(`data:image/png;base64,${base64_captcha}`);
      setPublicKey(public_key);
    } catch (e) {
      console.error("获取验证码失败: ", e);
      message.error("获取验证码失败");
    }
  };

  // 登录提交
  const onFinish: FormProps<FieldType>["onFinish"] = async (values) => {
    if (!publicKey) {
      message.error("公钥未加载，无法加密密码");
      return;
    }
    const encryptor = new JSEncrypt();
    encryptor.setPublicKey(publicKey);
    const encryptedPassword = encryptor.encrypt(values.password || "");
    if (!encryptedPassword) {
      message.error("密码加密失败");
      return;
    }
    try {
      const response = await axios.post("/api/login", {
        username: values.username,
        password: encryptedPassword,
        uuid: captchaUuid,
        captcha: values.captcha,
      });
      localStorage.setItem("username", String(values.username));
      const { token, restful_apis } = response.data;
      localStorage.setItem("token", token);
      localStorage.setItem("restful_apis", JSON.stringify(restful_apis));
      message.success("登录成功！");
      navigate("/");
    } catch (error: unknown) {
      console.error("Login failed:", error);
      message.error("登录失败，请检查账号、密码或验证码");
      getCaptcha();
    }
  };

  useEffect(() => {
    restful_api
      .get<{ base64_logo: string }>("/api/system/logo")
      .then((rsp) => {
        if (rsp.data.base64_logo) {
          setLogo(`data:image/png;base64,${rsp.data.base64_logo}`);
        }
      })
      .catch(console.error);
    getCaptcha();
  }, []);

  return (
    <div
      style={{
        height: "100vh",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: "linear-gradient(135deg, #f0f2f5 0%, #e6f7ff 100%)",
      }}
    >
      <Card
        style={{
          width: 400,
          padding: "24px 12px",
          borderRadius: 12,
          boxShadow: "0 4px 20px rgba(0,0,0,0.1)",
        }}
      >
        {/* Logo + 系统标题 */}
        <div style={{ textAlign: "center", marginBottom: 24 }}>
          <img src={logo} alt="Logo" style={{ width: 60, marginBottom: 12 }} />
          <h2 style={{ margin: 0 }}>管理系统登录</h2>
        </div>

        <Form
          name="login"
          size="large" // 控件统一大尺寸
          onFinish={onFinish}
          autoComplete="off"
          layout="vertical"
        >
          <Form.Item<FieldType>
            label="用户名"
            name="username"
            rules={[{ required: true, message: "请输入用户名" }]}
          >
            <Input placeholder="请输入用户名" />
          </Form.Item>

          <Form.Item<FieldType>
            label="密码"
            name="password"
            rules={[{ required: true, message: "请输入密码" }]}
          >
            <Input.Password placeholder="请输入密码" />
          </Form.Item>

          <Form.Item<FieldType>
            label="验证码"
            name="captcha"
            rules={[{ required: true, message: "请输入验证码" }]}
          >
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <Input
                placeholder="请输入验证码"
                style={{ flex: 1, height: 40 }}
                maxLength={6}
              />
              <img
                src={captchaImg}
                alt="captcha"
                title="点击刷新验证码"
                onClick={getCaptcha}
                draggable={false}
                style={{
                  width: 120,
                  height: 40,
                  objectFit: "contain",
                  cursor: "pointer",
                  borderRadius: 6,
                  border: "1px solid #d9d9d9",
                  background: "#fff",
                  userSelect: "none",
                }}
              />
            </div>
          </Form.Item>

          <Form.Item>
            <Button
              type="primary"
              htmlType="submit"
              block
              style={{ borderRadius: 6 }}
            >
              登录
            </Button>
          </Form.Item>
        </Form>
      </Card>
    </div>
  );
};

export default App;
