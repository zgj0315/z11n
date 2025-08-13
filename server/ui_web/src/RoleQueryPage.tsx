import React, { useEffect, useState } from "react";
import { Button, Form, Input, message, Table, Popconfirm } from "antd";
import restful_api from "./RESTfulApi.tsx";
import { useNavigate, Link } from "react-router-dom";

type RestfulApi = {
  method: string;
  name: string;
  path: string;
};

type Role = {
  id: number;
  name: string;
  restful_apis?: RestfulApi[];
};

type Page = {
  size: number;
  total_elements: number;
  total_pages: number;
};

const App: React.FC = () => {
  const navigate = useNavigate();
  const [roles, setRoles] = useState<Role[]>([]);
  const [page, setPage] = useState<Page | null>(null);
  const [current, setCurrent] = useState(1);
  const [page_size, setPageSize] = useState(5);
  const [loading, setLoading] = useState(false);
  const [isLoggedIn, setIsLoggedIn] = useState<boolean>(false);

  const handleQuery = async (
    page = current,
    size = page_size,
    filters?: { name?: string },
    showMessage = false
  ) => {
    const params = new URLSearchParams();
    params.append("size", size.toString());
    params.append("page", (page - 1).toString());
    if (filters?.name) params.append("name", filters.name);

    setLoading(true);
    try {
      const response = await restful_api.get(`/api/roles?${params.toString()}`);
      setRoles(response.data._embedded?.role || []);
      setPage(response.data.page);
      setCurrent(page);
      setPageSize(size);
      if (showMessage) message.success("查询成功");
    } catch (e) {
      console.error("查询失败: ", e);
      message.error("查询失败");
      navigate("/login");
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (id: number) => {
    try {
      await restful_api.delete(`/api/roles/${id}`);
      message.success("删除成功");

      // 如果当前页数据只剩 1 条并且不是第一页，则回到上一页
      if (roles.length === 1 && current > 1) {
        handleQuery(current - 1, page_size);
      } else {
        handleQuery();
      }
    } catch (error) {
      console.error("删除失败:", error);
      message.error("删除失败");
    }
  };
  const columns = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
    },
    {
      title: "角色名称",
      dataIndex: "name",
      key: "name",
    },
    {
      title: "操作",
      key: "action",
      render: (_: unknown, record: Role) => (
        <>
          <Button type="link" onClick={() => navigate(`/roles/${record.id}`)}>
            查看
          </Button>{" "}
          <Button
            type="link"
            onClick={() => navigate(`/roles/modify/${record.id}`)}
          >
            编辑
          </Button>
          {isLoggedIn && (
            <>
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
            </>
          )}
        </>
      ),
    },
  ];

  useEffect(() => {
    if (!roles.length) {
      const token = localStorage.getItem("token");
      setIsLoggedIn(!!token);
      handleQuery();
    }
  }, []);

  return (
    <>
      <Form
        layout="inline"
        onFinish={(values) => handleQuery(1, page_size, values)}
        style={{ marginTop: 16 }}
      >
        <Form.Item name="name" label="角色名称">
          <Input placeholder="请输入角色名称关键字" />
        </Form.Item>
        <Form.Item>
          <Button type="primary" htmlType="submit">
            查询
          </Button>
          <Button type="link">
            <Link to="/roles/create">创建角色</Link>
          </Button>
        </Form.Item>
      </Form>

      <Table
        dataSource={roles}
        columns={columns}
        rowKey="id"
        loading={loading}
        pagination={{
          current: current,
          pageSize: page_size,
          showSizeChanger: true,
          pageSizeOptions: ["5", "10", "20"],
          total: page?.total_elements,
          onChange: (page, size) => handleQuery(page, size),
        }}
        style={{ marginTop: 24 }}
      />
    </>
  );
};

export default App;
