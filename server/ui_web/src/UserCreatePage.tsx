import React, { useState, useEffect } from "react";
import { Form, Input, Button, Checkbox, message, Spin } from "antd";
import { useNavigate } from "react-router-dom";
import restful_api from "./RESTfulApi.tsx";

interface Role {
  id: number;
  name: string;
}

interface UserFormValues {
  username: string;
  password: string;
  role_ids: number[];
}

const UserCreate: React.FC = () => {
  const [form] = Form.useForm<UserFormValues>();
  const [roles, setRoles] = useState<Role[]>([]);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    const fetchRoles = async () => {
      try {
        const res = await restful_api.get("/api/roles?size=1000&page=0");
        setRoles(res.data._embedded?.role || []);
      } catch (err) {
        console.error("Failed to fetch roles:", err);
        message.error("获取角色列表失败");
      } finally {
        setLoading(false);
      }
    };

    fetchRoles();
  }, []);

  const onFinish = async (values: UserFormValues) => {
    setSubmitting(true);
    try {
      await restful_api.post("/api/users", values);
      message.success("用户创建成功");
      form.resetFields();
      navigate("/users");
    } catch (err) {
      console.error("Failed to create user:", err);
      message.error("用户创建失败");
    } finally {
      setSubmitting(false);
    }
  };

  if (loading) {
    return (
      <div style={{ display: "flex", justifyContent: "center", marginTop: 50 }}>
        <Spin tip="加载中..." />
      </div>
    );
  }

  return (
    <Form
      form={form}
      layout="vertical"
      onFinish={onFinish}
      style={{ maxWidth: 500, margin: "0 auto", marginTop: 24 }}
    >
      <Form.Item
        label="用户名"
        name="username"
        rules={[{ required: true, message: "请输入用户名" }]}
      >
        <Input placeholder="请输入用户名" />
      </Form.Item>

      <Form.Item
        label="密码"
        name="password"
        rules={[{ required: true, message: "请输入密码" }]}
      >
        <Input.Password placeholder="请输入密码" />
      </Form.Item>

      <Form.Item
        label="分配角色"
        name="role_ids"
        rules={[{ required: true, message: "请选择至少一个角色" }]}
      >
        <Checkbox.Group
          options={roles.map((r) => ({ label: r.name, value: r.id }))}
        />
      </Form.Item>

      <Form.Item>
        <Button type="primary" htmlType="submit" loading={submitting} block>
          创建用户
        </Button>
      </Form.Item>
    </Form>
  );
};

export default UserCreate;
