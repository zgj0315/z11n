import React, { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Form, Input, Button, Checkbox, message, Spin } from "antd";
import restful_api from "./utils/restful_api.ts";

interface Role {
  id: number;
  name: string;
}

interface UserData {
  id: number;
  username: string;
  roles: Role[];
}
interface UserFormValues {
  username: string;
  password: string;
  role_ids: number[];
}

const UserEdit: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [form] = Form.useForm();
  const [roles, setRoles] = useState<Role[]>([]);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    // 并行请求角色列表和用户数据
    Promise.all([
      restful_api.get("/api/roles?size=1000&page=0"),
      restful_api.get(`/api/users/${id}`),
    ])
      .then(([roleRes, userRes]) => {
        const allRoles: Role[] = roleRes.data._embedded.role;
        setRoles(allRoles);

        const user: UserData = userRes.data;
        // 回填表单
        form.setFieldsValue({
          username: user.username,
          role_ids: user.roles.map((r) => r.id),
        });
      })
      .catch((err) => {
        console.error("加载用户编辑数据失败:", err);
        message.error("加载数据失败");
      })
      .finally(() => {
        setLoading(false);
      });
  }, [id, form]);

  const onFinish = (values: UserFormValues) => {
    setSubmitting(true);
    restful_api
      .patch(`/api/users/${id}`, {
        username: values.username,
        // 如果密码为空，不传递该字段，避免覆盖
        ...(values.password ? { password: values.password } : {}),
        role_ids: values.role_ids || [],
      })
      .then(() => {
        message.success("用户更新成功");
        navigate("/users");
      })
      .catch((err) => {
        console.error("更新用户失败:", err);
        message.error("用户更新失败");
      })
      .finally(() => {
        setSubmitting(false);
      });
  };

  if (loading) {
    return <Spin tip="加载中..." />;
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

      <Form.Item label="密码" name="password" tooltip="留空则不修改密码">
        <Input.Password placeholder="如需修改请输入新密码" />
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
          保存修改
        </Button>
      </Form.Item>
    </Form>
  );
};

export default UserEdit;
