import React, { useEffect, useState } from "react";
import type { FormProps } from "antd";
import { Button, Form, Input, message, Row, Col } from "antd";
import axios from "axios";
import restful_api from "./utils/restful_api.ts";
import { useNavigate } from "react-router-dom";

type FieldType = {
  username?: string;
  password?: string;
  captcha?: string;
};

const App: React.FC = () => {
  const navigate = useNavigate();
  const [captchaImg, setCaptchaImg] = useState<string>(""); // 存 base64 验证码图片
  const [captchaUuid, setCaptchaUuid] = useState<string>(""); // 存 uuid

  const getCaptcha = async () => {
    try {
      const rsp = await restful_api.get("/api/captcha");
      const { uuid, base64_captcha } = rsp.data;
      setCaptchaUuid(uuid);
      setCaptchaImg(`data:image/png;base64,${base64_captcha}`);
    } catch (e) {
      console.error("获取验证码失败: ", e);
      message.error("获取验证码失败");
    }
  };

  const onFinish: FormProps<FieldType>["onFinish"] = async (values) => {
    try {
      const response = await axios.post("/api/login", {
        username: values.username,
        password: values.password,
        uuid: captchaUuid, // 传验证码对应的uuid
        captcha: values.captcha, // 用户输入的验证码
      });
      localStorage.setItem("username", String(values.username));
      const { token, restful_apis } = response.data;
      localStorage.setItem("token", token);
      localStorage.setItem("restful_apis", JSON.stringify(restful_apis));
      message.success("Login successful!");
      navigate("/");
    } catch (error: unknown) {
      console.error("Login failed:", error);
      message.error("Login failed. Please check your credentials.");
      getCaptcha(); // 登录失败后刷新验证码
    }
  };

  const onFinishFailed: FormProps<FieldType>["onFinishFailed"] = (
    errorInfo
  ) => {
    console.log("Failed:", errorInfo);
  };

  useEffect(() => {
    getCaptcha();
  }, []);

  return (
    <Form
      name="basic"
      labelCol={{ span: 8 }}
      wrapperCol={{ span: 16 }}
      style={{ maxWidth: 600 }}
      onFinish={onFinish}
      onFinishFailed={onFinishFailed}
      autoComplete="off"
    >
      <Form.Item<FieldType>
        label="Username"
        name="username"
        rules={[{ required: true, message: "Please input your username!" }]}
      >
        <Input />
      </Form.Item>

      <Form.Item<FieldType>
        label="Password"
        name="password"
        rules={[{ required: true, message: "Please input your password!" }]}
      >
        <Input.Password />
      </Form.Item>

      <Form.Item<FieldType>
        label="Captcha"
        name="captcha"
        rules={[{ required: true, message: "Please input captcha!" }]}
      >
        <Row gutter={8}>
          <Col span={12}>
            <Input placeholder="Enter captcha" />
          </Col>
          <Col span={12}>
            <img
              src={captchaImg}
              alt="captcha"
              style={{ cursor: "pointer", height: 32 }}
              onClick={getCaptcha} // 点击刷新验证码
            />
          </Col>
        </Row>
      </Form.Item>

      <Form.Item wrapperCol={{ offset: 8 }}>
        <Button type="primary" htmlType="submit">
          Submit
        </Button>
      </Form.Item>
    </Form>
  );
};

export default App;
