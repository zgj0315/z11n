import React, { useEffect, useState } from "react";
import { Button, Form, Input, message, Table, Popconfirm, Upload } from "antd";
import restful_api from "./RESTfulApi.tsx";
import dayjs from "dayjs";
import { UploadOutlined } from "@ant-design/icons";
import type { UploadProps } from "antd";
import { useNavigate } from "react-router-dom";

type Host = {
  agent_id: string;
};

type Page = {
  size: number;
  total_elements: number;
  total_pages: number;
};

const App: React.FC = () => {
  const navigate = useNavigate();
  const [hosts, setHosts] = useState<[]>([]);
  const [current, setCurrent] = useState(1);
  const [page_size, setPageSize] = useState(5);
  const [page, setPage] = useState<Page>();
  const [loading, setLoading] = useState(false);
  const [isLoggedIn, setIsLoggedIn] = useState<boolean>(false);

  const create_props: UploadProps = {
    name: "file",
    showUploadList: false,
    customRequest: async (options) => {
      const { file, onSuccess, onProgress } = options;

      const formData = new FormData();
      formData.append("file", file as Blob);

      try {
        const response = await restful_api.post("/api/hosts", formData, {
          onUploadProgress: (event) => {
            if (event.total) {
              const percent = Math.round((event.loaded * 100) / event.total);
              onProgress?.({ percent });
            }
          },
        });
        onSuccess?.(response.data);
        message.success(`${(file as File).name} uploaded successfully`);
        handleQuery();
      } catch (error) {
        console.error("Upload error:", error);
        message.error(`${(file as File).name} upload failed.`);
      }
    },
  };
  const update_props = (id: string): UploadProps => ({
    name: "file",
    showUploadList: false,
    customRequest: async (options) => {
      const { file, onSuccess, onProgress } = options;

      const formData = new FormData();
      formData.append("file", file as Blob);

      try {
        const response = await restful_api.patch(`/api/hosts/${id}`, formData, {
          onUploadProgress: (event) => {
            if (event.total) {
              const percent = Math.round((event.loaded * 100) / event.total);
              onProgress?.({ percent });
            }
          },
        });
        onSuccess?.(response.data);
        message.success(`${(file as File).name} uploaded successfully`);
        handleQuery();
      } catch (error) {
        console.error("Upload error:", error);
        message.error(`${(file as File).name} upload failed.`);
      }
    },
  });

  const handleQuery = async (
    page = current,
    size = page_size,
    filters?: { title?: string; content?: string }
  ) => {
    console.log("handleQuery page: ", page);
    console.log("handleQuery size: ", size);
    const params = new URLSearchParams();
    params.append("size", size.toString());
    params.append("page", (page - 1).toString());
    if (filters?.title) params.append("title", filters.title);
    setLoading(true);
    try {
      const response = await restful_api.get(`/api/hosts?${params.toString()}`);
      setHosts(response.data._embedded?.host);
      setPage(response.data.page);
      setCurrent(page);
      setPageSize(size);
      message.success("查询成功");
    } catch (e) {
      console.error("查询失败: ", e);
      message.error("查询失败");
    } finally {
      setLoading(false);
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
    // {
    //   title: "AgentId",
    //   dataIndex: "agent_id",
    //   key: "agent_id",
    // },
    {
      title: "主机名",
      dataIndex: "host_name",
      key: "host_name",
    },
    {
      title: "系统名称",
      dataIndex: "name",
      key: "name",
    },
    {
      title: "系统版本",
      dataIndex: "os_version",
      key: "os_version",
    },
    {
      title: "CPU架构",
      dataIndex: "cpu_arch",
      key: "cpu_arch",
    },
    {
      title: "创建时间",
      dataIndex: "created_at",
      key: "created_at",
      render: (timestamp: number) =>
        timestamp ? dayjs(timestamp).format("YYYY-MM-DD HH:mm:ss") : "--",
    },
    {
      title: "修改时间",
      dataIndex: "updated_at",
      key: "updated_at",
      render: (timestamp: number) =>
        timestamp ? dayjs(timestamp).format("YYYY-MM-DD HH:mm:ss") : "--",
    },
    {
      title: "操作",
      key: "action",
      render: (_: unknown, record: Host) => (
        <>
          <Button
            type="link"
            onClick={() => navigate(`/hosts/${record.agent_id}`)}
          >
            查看
          </Button>
          {isLoggedIn && (
            <>
              <Upload {...update_props(record.agent_id)}>
                <Button danger type="link">
                  编辑
                </Button>
              </Upload>
              <Popconfirm
                title="确定要删除这条记录吗？"
                onConfirm={() => handleDelete(record.agent_id)}
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
        <Form.Item name="title" label="标题">
          <Input placeholder="请输入标题关键字" />
        </Form.Item>
        <Form.Item>
          <Button type="primary" htmlType="submit">
            查询
          </Button>
        </Form.Item>
        {isLoggedIn && (
          <Form.Item>
            <Upload {...create_props}>
              <Button icon={<UploadOutlined />}>上传文件</Button>
            </Upload>
          </Form.Item>
        )}
      </Form>

      <Table
        dataSource={hosts}
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
