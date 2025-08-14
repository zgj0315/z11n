import React, { useEffect, useState, useCallback } from "react";
import { Button, Form, Input, message, Table, Popconfirm, Space } from "antd";
import restful_api from "./RESTfulApi.tsx";
import dayjs from "dayjs";
import { useNavigate } from "react-router-dom";
import { hasPermission } from "./utils/permission";

type Host = {
  id: string;
  agent_id: string;
  host_name: string;
  name: string;
  os_version: string;
  cpu_arch: string;
  created_at?: number;
  updated_at?: number;
};

type Page = {
  size: number;
  total_elements: number;
  total_pages: number;
};

const formatDate = (timestamp?: number) =>
  timestamp ? dayjs(timestamp).format("YYYY-MM-DD HH:mm:ss") : "--";

const App: React.FC = () => {
  const navigate = useNavigate();
  const [hosts, setHosts] = useState<Host[]>([]);
  const [current, setCurrent] = useState(1);
  const [pageSize, setPageSize] = useState(5);
  const [page, setPage] = useState<Page>({
    size: 5,
    total_elements: 0,
    total_pages: 0,
  });
  const [loading, setLoading] = useState(false);

  const handleQuery = useCallback(
    async (
      pageNum = current,
      size = pageSize,
      filters?: { title?: string }
    ) => {
      const params = new URLSearchParams();
      params.append("size", size.toString());
      params.append("page", (pageNum - 1).toString());
      if (filters?.title) params.append("title", filters.title);

      setLoading(true);
      try {
        const response = await restful_api.get(
          `/api/hosts?${params.toString()}`
        );
        setHosts(response.data._embedded?.host || []);
        setPage(
          response.data.page || { size, total_elements: 0, total_pages: 0 }
        );
        setCurrent(pageNum);
        setPageSize(size);
        message.success("查询成功");
      } catch (error) {
        console.error("查询失败: ", error);
        message.error("查询失败");
      } finally {
        setLoading(false);
      }
    },
    [current, pageSize]
  );

  const handleUpload = async (record: Host) => {
    try {
      await restful_api.post("/api/hosts", record);
      message.success("主机信息更新成功");
    } catch (error) {
      console.error("更新失败:", error);
      message.error("主机信息更新失败");
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await restful_api.delete(`/api/hosts/${id}`);
      message.success("删除成功");
      handleQuery();
    } catch (error) {
      console.error("删除失败:", error);
      message.error("删除失败");
    }
  };

  const columns = [
    { title: "主机名", dataIndex: "host_name", key: "host_name" },
    { title: "系统名称", dataIndex: "name", key: "name" },
    { title: "系统版本", dataIndex: "os_version", key: "os_version" },
    { title: "CPU架构", dataIndex: "cpu_arch", key: "cpu_arch" },
    {
      title: "创建时间",
      dataIndex: "created_at",
      key: "created_at",
      render: formatDate,
    },
    {
      title: "修改时间",
      dataIndex: "updated_at",
      key: "updated_at",
      render: formatDate,
    },
    {
      title: "操作",
      key: "action",
      render: (_: unknown, record: Host) => (
        <Space size="middle">
          {hasPermission("GET", "/api/hosts/") && (
            <Button
              type="link"
              onClick={() => navigate(`/hosts/${record.agent_id}`)}
            >
              查看
            </Button>
          )}
          {hasPermission("POST", "/api/hosts") && (
            <Popconfirm
              title="确定要更新这条记录吗？"
              onConfirm={() => handleUpload(record)}
              okText="确定"
              cancelText="取消"
            >
              <Button type="link" danger>
                更新
              </Button>
            </Popconfirm>
          )}
          {hasPermission("DELETE", "/api/agents/") && (
            <Popconfirm
              title="确定要删除这条记录吗？"
              onConfirm={() => handleDelete(record.agent_id)}
              okText="确定"
              cancelText="取消"
            >
              <Button type="link" danger>
                删除
              </Button>
            </Popconfirm>
          )}
        </Space>
      ),
    },
  ];

  useEffect(() => {
    handleQuery();
  }, [handleQuery]);

  return (
    <>
      <Form
        layout="inline"
        onFinish={(values) => handleQuery(1, pageSize, values)}
        style={{ marginTop: 16 }}
      >
        <Form.Item name="title" label="标题">
          <Input placeholder="请输入标题关键字" />
        </Form.Item>
        <Form.Item>
          <Button type="primary" htmlType="submit">
            查询
          </Button>
        </Form.Item>
      </Form>

      <Table
        dataSource={hosts}
        columns={columns}
        rowKey="id"
        loading={loading}
        pagination={{
          current,
          pageSize,
          total: page.total_elements,
          onChange: (pageNum, size) => handleQuery(pageNum, size),
        }}
        style={{ marginTop: 24 }}
      />
    </>
  );
};

export default App;
