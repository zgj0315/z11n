import React, { useEffect, useState, useCallback, useMemo } from "react";
import { Button, Form, Input, message, Table, Popconfirm } from "antd";
import { useNavigate, Link } from "react-router-dom";
import restful_api from "./utils/restful_api.ts";
import { hasPermission } from "./utils/permission";

interface User {
  id: number;
  username: string;
}

interface PaginationState {
  current: number;
  pageSize: number;
  total: number;
}

const App: React.FC = () => {
  const navigate = useNavigate();
  const [users, setUsers] = useState<User[]>([]);
  const [pagination, setPagination] = useState<PaginationState>({
    current: 1,
    pageSize: 5,
    total: 0,
  });
  const [loading, setLoading] = useState(false);

  const fetchUsers = useCallback(
    async (
      page: number = pagination.current,
      size: number = pagination.pageSize,
      filters?: { username?: string }
    ) => {
      setLoading(true);
      try {
        const params = new URLSearchParams({
          size: size.toString(),
          page: (page - 1).toString(),
          ...(filters?.username ? { username: filters.username } : {}),
        });

        const { data } = await restful_api.get(
          `/api/users?${params.toString()}`
        );
        setUsers(data._embedded?.user || []);
        setPagination({
          current: page,
          pageSize: size,
          total: data.page?.total_elements || 0,
        });
      } catch (error) {
        console.error("查询失败:", error);
        message.error("查询失败");
      } finally {
        setLoading(false);
      }
    },
    [pagination.current, pagination.pageSize, navigate]
  );

  const handleDelete = useCallback(
    async (id: number) => {
      try {
        await restful_api.delete(`/api/users/${id}`);
        message.success("删除成功");
        fetchUsers(); // 保持当前页
      } catch (error) {
        console.error("删除失败:", error);
        message.error("删除失败");
      }
    },
    [fetchUsers]
  );

  const columns = useMemo(
    () => [
      { title: "ID", dataIndex: "id", key: "id" },
      { title: "用户名", dataIndex: "username", key: "username" },
      {
        title: "操作",
        key: "action",
        render: (_: unknown, record: User) => (
          <>
            {hasPermission("GET", "/api/users") && (
              <Button
                type="link"
                onClick={() => navigate(`/users/${record.id}`)}
              >
                查看
              </Button>
            )}
            {hasPermission("PATCH", "/api/users/") && (
              <Button
                type="link"
                onClick={() => navigate(`/users/modify/${record.id}`)}
              >
                编辑
              </Button>
            )}
            {hasPermission("DELETE", "/api/users/") && (
              <Popconfirm
                title="确定要删除这条记录吗？"
                onConfirm={() => handleDelete(record.id)}
                okText="确定"
                cancelText="取消"
              >
                <Button danger type="link">
                  删除
                </Button>
              </Popconfirm>
            )}
          </>
        ),
      },
    ],
    [navigate, handleDelete]
  );

  useEffect(() => {
    fetchUsers();
  }, [fetchUsers]);

  return (
    <>
      <Form
        layout="inline"
        onFinish={(values) => fetchUsers(1, pagination.pageSize, values)}
        style={{ marginTop: 16 }}
      >
        <Form.Item name="username" label="用户名">
          <Input placeholder="请输入用户名关键字" allowClear />
        </Form.Item>
        <Form.Item>
          <Button type="primary" htmlType="submit">
            查询
          </Button>
          {hasPermission("POST", "/api/users") && (
            <Button type="link">
              <Link to="/users/create">创建用户</Link>
            </Button>
          )}
        </Form.Item>
      </Form>

      <Table
        dataSource={users}
        columns={columns}
        rowKey="id"
        loading={loading}
        pagination={{
          current: pagination.current,
          pageSize: pagination.pageSize,
          total: pagination.total,
          onChange: (page, size) => fetchUsers(page, size),
        }}
        style={{ marginTop: 24 }}
      />
    </>
  );
};

export default App;
