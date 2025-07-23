import React from "react";
import type { FormProps } from "antd";
import { Button, Form, Input, message } from "antd";
import axios from "axios";

type FieldType = {
  username?: string;
  password?: string;
  remember?: string;
};

const onFinish: FormProps<FieldType>["onFinish"] = async (values) => {
  console.log("Success:", values);
  try {
    const response = await axios.post("/api/login", {
      username: values.username,
      password: values.password,
    });

    const { token } = response.data;
    console.log("token: ", token);
    localStorage.setItem("token", token);
    message.success("Login successful!");
    window.location.href = "/";
  } catch (error: unknown) {
    console.error("Login failed:", error);
  }
};

const onFinishFailed: FormProps<FieldType>["onFinishFailed"] = (errorInfo) => {
  console.log("Failed:", errorInfo);
};

const App: React.FC = () => (
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

    <Form.Item label={null}>
      <Button type="primary" htmlType="submit">
        Submit
      </Button>
    </Form.Item>
  </Form>
);

export default App;
