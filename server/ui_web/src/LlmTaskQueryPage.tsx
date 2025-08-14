import React, { useEffect, useState } from "react";
import { Button, Form, Input, message, Table, Popconfirm } from "antd";
import restful_api from "./RESTfulApi.tsx";
import dayjs from "dayjs";
import { useNavigate } from "react-router-dom";

type LlmTask = {
  id: string;
};

type Page = {
  size: number;
  total_elements: number;
  total_pages: number;
};

const App: React.FC = () => {
  const navigate = useNavigate();
  const [llmTasks, setLlmTasks] = useState<[]>([]);
  const [current, setCurrent] = useState(1);
  const [page_size, setPageSize] = useState(5);
  const [page, setPage] = useState<Page>();
  const [loading, setLoading] = useState(false);
  const [isLoggedIn, setIsLoggedIn] = useState<boolean>(false);

  const handleQuery = async (
    page = current,
    size = page_size,
    filters?: { title?: string; content?: string }
  ) => {
    const params = new URLSearchParams();
    params.append("size", size.toString());
    params.append("page", (page - 1).toString());
    if (filters?.title) params.append("title", filters.title);
    setLoading(true);
    try {
      const response = await restful_api.get(
        `/api/llm_tasks?${params.toString()}`
      );
      setLlmTasks(response.data._embedded?.llm_task);
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
      await restful_api.delete(`/api/llm_tasks/${id}`);
      message.success("删除成功");
      handleQuery();
    } catch (error) {
      console.error("删除失败:", error);
      message.error("删除失败");
    }
  };
  const columns = [
    {
      title: "TaskId",
      dataIndex: "id",
      key: "id",
    },
    {
      title: "Model",
      dataIndex: "model",
      key: "model",
    },
    {
      title: "Prompt",
      dataIndex: "prompt",
      key: "prompt",
    },
    {
      title: "问题",
      dataIndex: "req_content",
      key: "req_content",
    },
    {
      title: "问题创建时间",
      dataIndex: "req_push_at",
      key: "req_push_at",
      render: (timestamp: number) =>
        timestamp ? dayjs(timestamp).format("YYYY-MM-DD HH:mm:ss") : "--",
    },
    {
      title: "问题处理时间",
      dataIndex: "req_pull_at",
      key: "req_pull_at",
      render: (timestamp: number) =>
        timestamp ? dayjs(timestamp).format("YYYY-MM-DD HH:mm:ss") : "--",
    },
    {
      title: "答案",
      dataIndex: "rsp_content",
      key: "rsp_content",
    },
    {
      title: "答案处理时间",
      dataIndex: "rsp_push_at",
      key: "rsp_push_at",
      render: (timestamp: number) =>
        timestamp ? dayjs(timestamp).format("YYYY-MM-DD HH:mm:ss") : "--",
    },
    {
      title: "答案提取时间",
      dataIndex: "rsp_pull_at",
      key: "rsp_pull_at",
      render: (timestamp: number) =>
        timestamp ? dayjs(timestamp).format("YYYY-MM-DD HH:mm:ss") : "--",
    },
    {
      title: "操作",
      key: "action",
      render: (_: unknown, record: LlmTask) => (
        <>
          <Button
            type="link"
            onClick={() => navigate(`/llm_tasks/${record.id}`)}
          >
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
        dataSource={llmTasks}
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
