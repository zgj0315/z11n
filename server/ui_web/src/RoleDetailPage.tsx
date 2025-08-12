import React, { useState, useEffect } from "react";
import { useParams } from "react-router-dom";
import { Descriptions, Spin, Transfer } from "antd";
import type { DescriptionsProps, TransferProps } from "antd";
import restful_api from "./RESTfulApi.tsx";

interface ApiRecord {
  key: string;
  title: string;
  description: string;
}

const App: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [items, setItems] = useState<DescriptionsProps["items"]>([]);
  const [loading, setLoading] = useState(true);

  const [apiData, setApiData] = useState<ApiRecord[]>([]);
  const [targetKeys, setTargetKeys] = useState<TransferProps["targetKeys"]>([]);
  const [selectedKeys, setSelectedKeys] = useState<TransferProps["targetKeys"]>(
    []
  );
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

  useEffect(() => {
    restful_api
      .get(`/api/roles/${id}`)
      .then((res) => {
        const data = res.data;
        const baseItems: DescriptionsProps["items"] = [
          {
            key: "id",
            label: "ID",
            children: data.id,
          },
          {
            key: "name",
            label: "角色名称",
            children: data.name,
          },
        ];
        setItems(baseItems);
        const transferData: ApiRecord[] = data.apis.map(
          (api: unknown, index: number) => ({
            key: String(index),
            title: `${api.restful_api.name}`,
            description: `${api.restful_api.method} ${api.restful_api.path}`,
          })
        );
        const ownedKeys = data.apis
          .map((api: unknown, index: number) =>
            api.is_owned ? String(index) : null
          )
          .filter((key: string | null) => key !== null) as string[];

        setApiData(transferData);
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

  if (!items) {
    return <div>No data</div>;
  }
  return (
    <>
      <Descriptions title="Role Info" bordered items={items} />
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
    </>
  );
};

export default App;
