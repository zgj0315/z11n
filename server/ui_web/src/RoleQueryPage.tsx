import React, { useEffect, useState } from "react";
import { Button, Form, Input, message, Table, Popconfirm } from "antd";
import restful_api from "./RESTfulApi.tsx";
import { useNavigate, Link } from "react-router-dom";

type Agent = {
  id: string;
};

type Page = {
  size: number;
  total_elements: number;
  total_pages: number;
};

const App: React.FC = () => {
  const navigate = useNavigate();
  const [roles, setRoles] = useState<[]>([]);
  const [current, setCurrent] = useState(1);
  const [page_size, setPageSize] = useState(5);
  const [page, setPage] = useState<Page>();
  const [loading, setLoading] = useState(false);
  const [isLoggedIn, setIsLoggedIn] = useState<boolean>(false);

  const handleQuery = async (
    page = current,
    size = page_size,
    filters?: { name?: string }
  ) => {
    console.log("handleQuery page: ", page);
    console.log("handleQuery size: ", size);
    const params = new URLSearchParams();
    params.append("size", size.toString());
    params.append("page", (page - 1).toString());
    if (filters?.name) params.append("name", filters.name);
    setLoading(true);
    try {
      const response = await restful_api.get(`/api/roles?${params.toString()}`);
      setRoles(response.data._embedded?.role);
      setPage(response.data.page);
      setCurrent(page);
      setPageSize(size);
      message.success("查询成功");
    } catch (e) {
      console.error("查询失败: ", e);
      message.error("查询失败");
      window.location.href = "/login";
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await restful_api.delete(`/api/roles/${id}`);
      message.success("删除成功");
      handleQuery();
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
      render: (_: unknown, record: Agent) => (
        <>
          <Button type="link" onClick={() => navigate(`/roles/${record.id}`)}>
            查看
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
    const token = localStorage.getItem("token");
    setIsLoggedIn(!!token);
    handleQuery();
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
          <Button type="link"><Link to="/roles/create">创建角色</Link></Button>
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
          total: page?.total_elements,
          onChange: (page, size) => handleQuery(page, size),
        }}
        style={{ marginTop: 24 }}
      />
    </>
  );
};

export default App;
