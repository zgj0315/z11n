import React, { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Input, Spin, Transfer, message, Button, Form } from "antd";
import type { TransferProps } from "antd";
import restful_api from "./utils/restful_api.ts";

interface RestfulApi {
  method: string;
  path: string;
  name: string;
}

interface ApiRecord extends RestfulApi {
  key: string;
  title: string;
  description: string;
}

interface Role {
  id: number;
  name: string;
  restful_apis?: RestfulApi[];
}

const App: React.FC = () => {
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();
  const [loading, setLoading] = useState(true);
  const [apiData, setApiData] = useState<ApiRecord[]>([]);
  const [targetKeys, setTargetKeys] = useState<string[]>([]);
  const [selectedKeys, setSelectedKeys] = useState<string[]>([]);
  const [roleName, setRoleName] = useState("");
  const onChange: TransferProps["onChange"] = (
    nextTargetKeys,
    direction,
    moveKeys
  ) => {
    console.log("targetKeys:", nextTargetKeys);
    console.log("direction:", direction);
    console.log("moveKeys:", moveKeys);
    setTargetKeys(nextTargetKeys as string[]);
  };
  const onSelectChange: TransferProps["onSelectChange"] = (
    sourceSelectedKeys,
    targetSelectedKeys
  ) => {
    console.log("sourceSelectedKeys:", sourceSelectedKeys);
    console.log("targetSelectedKeys:", targetSelectedKeys);
    setSelectedKeys([
      ...sourceSelectedKeys.map(String),
      ...targetSelectedKeys.map(String),
    ]);
  };

  const onScroll: TransferProps["onScroll"] = (direction, e) => {
    console.log("direction:", direction);
    console.log("target:", e.target);
  };
  const [saving, setSaving] = useState(false);

  const handleSubmit = async () => {
    if (!roleName.trim()) {
      return message.error("请输入角色名称");
    }
    if (targetKeys.length === 0) {
      return message.error("请选择至少一个 API");
    }

    const payload = {
      name: roleName.trim(),
      restful_apis: apiData
        .filter((api) => targetKeys.includes(api.key))
        .map(({ method, name, path }) => ({ method, name, path })),
    };

    try {
      setSaving(true);
      await restful_api.patch(`/api/roles/${id}`, payload);
      message.success("角色修改成功");
      navigate("/roles");
    } catch (err) {
      console.error("提交失败:", err);
      message.error("提交失败，请稍后重试");
    } finally {
      setSaving(false);
    }
  };
  useEffect(() => {
    const loadData = async () => {
      setLoading(true);
      try {
        const [apiRes, roleRes] = await Promise.all([
          restful_api.get<RestfulApi[]>("/api/restful_apis"),
          restful_api.get<Role>(`/api/roles/${id}`),
        ]);

        const transferData: ApiRecord[] = (apiRes.data || []).map((api) => ({
          key: `${api.method}:${api.path.trim()}`,
          title: api.name,
          description: `${api.method} ${api.path}`,
          ...api,
        }));

        setApiData(transferData);
        setRoleName(roleRes.data.name || "");

        const ownedKeys = (roleRes.data.restful_apis || []).map(
          (api) => `${api.method}:${api.path.trim()}`
        );
        setTargetKeys(ownedKeys);
      } catch (err) {
        console.error("加载数据失败", err);
        message.error("加载数据失败，请稍后重试");
      } finally {
        setLoading(false);
      }
    };

    loadData();
  }, [id]);

  if (loading) {
    return <Spin tip="Loading..." />;
  }

  return (
    <>
      <Form onFinish={handleSubmit}>
        <Form.Item
          label="角色名称"
          rules={[{ required: true, message: "请输入角色名称" }]}
        >
          <Input
            value={roleName}
            onChange={(e) => setRoleName(e.target.value)}
          />
        </Form.Item>
      </Form>
      <Transfer
        showSearch
        filterOption={(inputValue, item) =>
          item?.title?.toLowerCase().includes(inputValue.toLowerCase()) ||
          item?.description?.toLowerCase().includes(inputValue.toLowerCase())
        }
        dataSource={apiData}
        titles={["未授权 API", "已授权 API"]}
        targetKeys={targetKeys}
        selectedKeys={selectedKeys}
        onChange={onChange}
        onSelectChange={onSelectChange}
        onScroll={onScroll}
        render={(item) => (
          <>
            <b>{item.title}</b>
            <div style={{ color: "#999" }}>{item.description}</div>
          </>
        )}
        listStyle={{ width: 300, height: 400 }}
      />
      <Button
        type="primary"
        onClick={handleSubmit}
        style={{ marginTop: 16 }}
        loading={saving}
        disabled={saving}
      >
        保存修改
      </Button>
    </>
  );
};

export default App;
