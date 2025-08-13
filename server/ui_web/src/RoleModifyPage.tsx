import React, { useState, useEffect } from "react";
import { useParams } from "react-router-dom";
import { Input, Spin, Transfer, message, Button } from "antd";
import type { TransferProps } from "antd";
import restful_api from "./RESTfulApi.tsx";

interface ApiRecord {
  key: string;
  title: string;
  description: string;
  method: string;
  name: string;
  path: string;
}

interface RestfulApi {
  method: string;
  path: string;
  name: string;
}

const App: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  // const [items, setItems] = useState<DescriptionsProps["items"]>([]);
  const [loading, setLoading] = useState(true);

  const [apiData, setApiData] = useState<ApiRecord[]>([]);
  const [targetKeys, setTargetKeys] = useState<TransferProps["targetKeys"]>([]);
  const [selectedKeys, setSelectedKeys] = useState<TransferProps["targetKeys"]>(
    []
  );
  const [roleName, setRoleName] = useState("");
  const onChange: TransferProps["onChange"] = (
    nextTargetKeys,
    direction,
    moveKeys
  ) => {
    console.log("targetKeys:", nextTargetKeys);
    console.log("direction:", direction);
    console.log("moveKeys:", moveKeys);
    setTargetKeys(nextTargetKeys);
  };
  const onSelectChange: TransferProps["onSelectChange"] = (
    sourceSelectedKeys,
    targetSelectedKeys
  ) => {
    console.log("sourceSelectedKeys:", sourceSelectedKeys);
    console.log("targetSelectedKeys:", targetSelectedKeys);
    setSelectedKeys([...sourceSelectedKeys, ...targetSelectedKeys]);
  };

  const onScroll: TransferProps["onScroll"] = (direction, e) => {
    console.log("direction:", direction);
    console.log("target:", e.target);
  };
  const handleSubmit = async () => {
    if (!roleName.trim()) {
      message.error("请输入角色名称");
      return;
    }

    const selectedApis = apiData.filter((api) => targetKeys.includes(api.key));
    if (selectedApis.length === 0) {
      message.error("请选择至少一个 API");
      return;
    }

    console.log("selectedApis: ", selectedApis);
    const payload = {
      name: roleName.trim(),
      restful_apis: selectedApis.map(({ method, name, path }) => ({
        method,
        name,
        path,
      })),
    };
    console.log("payload: ", payload);

    try {
      await restful_api.patch(`/api/roles/${id}`, payload);
      message.success("角色修改成功");
      window.location.href = "/roles";
    } catch (err) {
      console.error("提交失败:", err);
      message.error("提交失败，请稍后重试");
    }
  };
  useEffect(() => {
    restful_api
      .get(`/api/restful_apis`)
      .then((res) => {
        const transferData: ApiRecord[] = res.data.map(
          (restful_api: RestfulApi) => ({
            key: `${restful_api.method}-${restful_api.path}`,
            title: `${restful_api.name}`,
            description: `${restful_api.method} ${restful_api.path}`,
            method: restful_api.method,
            name: restful_api.name,
            path: restful_api.path,
          })
        );
        // console.log("transferData: ", transferData);
        setApiData(transferData);
      })
      .catch((err) => {
        console.error("Failed to fetch system info:", err);
      })
      .finally(() => {});

    restful_api
      .get(`/api/roles/${id}`)
      .then((res) => {
        setRoleName(res.data.name || "");
        const ownedKeys = res.data.restful_apis.map(
          (api: RestfulApi) => `${api.method}-${api.path}`
        );
        // console.log("ownedKeys: ", ownedKeys);
        setTargetKeys(ownedKeys);
      })
      .catch((err) => {
        console.error("Failed to fetch system info:", err);
      })
      .finally(() => {
        setLoading(false);
      });
  }, []);

  if (loading) {
    return <Spin tip="Loading..." />;
  }

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
      <Button type="primary" onClick={handleSubmit} style={{ marginTop: 16 }}>
        保存修改
      </Button>
    </>
  );
};

export default App;
