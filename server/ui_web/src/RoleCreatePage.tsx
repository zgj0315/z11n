import React, { useState, useEffect } from "react";
import { Transfer, message, Input, Button } from "antd";
import type { TransferProps } from "antd";
import restful_api from "./utils/restful_api.ts";
import { useNavigate } from "react-router-dom";
import type { RestfulApi } from "./types/restfulApi.ts";

interface ApiRecord {
  key: string;
  title: string;
  description: string;
  method: string;
  name: string;
  path: string;
}

const App: React.FC = () => {
  const navigate = useNavigate();
  const [apiData, setApiData] = useState<ApiRecord[]>([]);
  const [targetKeys, setTargetKeys] = useState<string[]>([]);
  const [selectedKeys, setSelectedKeys] = useState<string[]>([]);
  const [roleName, setRoleName] = useState("");

  const onChange: TransferProps<ApiRecord>["onChange"] = (nextTargetKeys) => {
    setTargetKeys(nextTargetKeys as string[]);
  };
  const onSelectChange: TransferProps<ApiRecord>["onSelectChange"] = (
    sourceSelectedKeys,
    targetSelectedKeys
  ) => {
    setSelectedKeys([
      ...sourceSelectedKeys.map(String),
      ...targetSelectedKeys.map(String),
    ]);
  };
  const onScroll: TransferProps["onScroll"] = (direction, e) => {
    console.log("direction:", direction);
    console.log("target:", e.target);
  };
  const handleSubmit = async () => {
    console.log("handleSubmit");
    if (!roleName.trim()) {
      console.error("请输入角色名称");
      message.error("请输入角色名称");
      return;
    }
    console.log("roleName: ", roleName);

    const selectedApis = apiData.filter((api) => targetKeys.includes(api.key));
    console.log("selectedApis: ", selectedApis);

    if (selectedApis.length === 0) {
      console.error("请选择至少一个 API");
      message.error("请选择至少一个 API");
      return;
    }

    const payload = {
      name: roleName.trim(),
      restful_apis: selectedApis.map(({ method, name, path }) => ({
        method,
        name,
        path,
      })),
    };

    console.log("提交数据:", payload);

    try {
      await restful_api.post("/api/roles", payload);
      message.success("角色创建成功");
      console.log("角色创建成功");

      // 重置表单
      setRoleName("");
      setTargetKeys([]);
      setSelectedKeys([]);
      navigate("/roles");
    } catch (err) {
      console.error("提交失败:", err);
      message.error("提交失败，请稍后重试");
    }
  };

  useEffect(() => {
    restful_api
      .get(`/api/restful_apis`)
      .then((res) => {
        const transferData: ApiRecord[] = res.data.map((api: RestfulApi) => ({
          key: `${api.method}-${api.path}`.trim(),
          title: api.name,
          description: `${api.method} ${api.path}`,
          method: api.method,
          name: api.name,
          path: api.path,
        }));
        setApiData(transferData);
      })
      .catch((err) => {
        console.error("Failed to fetch system info:", err);
        message.error("加载 API 列表失败");
      })
      .finally(() => {});
  }, []);

  return (
    <>
      <Input
        placeholder="请输入角色名称"
        value={roleName}
        onChange={(e) => setRoleName(e.target.value)}
        style={{ width: 300, marginBottom: 16 }}
      />
      <Transfer
        dataSource={apiData}
        titles={["未授权的API", "已授权的API"]}
        targetKeys={targetKeys}
        selectedKeys={selectedKeys}
        onChange={onChange}
        onSelectChange={onSelectChange}
        onScroll={onScroll}
        render={(item) => `${item.title} - ${item.description}`}
        listStyle={{
          width: 300,
          height: 400,
        }}
      />
      <Button type="primary" onClick={handleSubmit}>
        提交
      </Button>
    </>
  );
};

export default App;
