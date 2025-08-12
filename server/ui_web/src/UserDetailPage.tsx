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

  const onChange: TransferProps["onChange"] = (nextTargetKeys) => {
    setTargetKeys(nextTargetKeys);
  };

  const onSelectChange: TransferProps["onSelectChange"] = (
    sourceSelectedKeys,
    targetSelectedKeys
  ) => {
    setSelectedKeys([...sourceSelectedKeys, ...targetSelectedKeys]);
  };

  useEffect(() => {
    restful_api
      .get(`/api/users/${id}`)
      .then((res) => {
        const data = res.data;

        // 用户信息
        const baseItems: DescriptionsProps["items"] = [
          { key: "id", label: "ID", children: data.id },
          { key: "name", label: "用户名", children: data.name },
        ];
        setItems(baseItems);

        // 遍历所有角色的 restful_apis
        // const allApis: ApiRecord[] = [];
        // const ownedKeys: string[] = [];
        const apiMap = new Map<string, ApiRecord>();
        const ownedKeysSet = new Set<string>();

        data.roles.forEach((role) => {
          role.restful_apis.forEach((api) => {
            const key = `${api.restful_api.method}-${api.restful_api.path}`;
            if (!apiMap.has(key)) {
              apiMap.set(key, {
                key,
                title: api.restful_api.name,
                description: `${api.restful_api.method} ${api.restful_api.path}`,
              });
            }
            if (api.is_owned) {
              ownedKeysSet.add(key);
            }
          });
        });

        setApiData(Array.from(apiMap.values()));
        setTargetKeys(Array.from(ownedKeysSet));
      })
      .catch((err) => {
        console.error("Failed to fetch system info:", err);
      })
      .finally(() => {
        setLoading(false);
      });
  }, [id]);

  if (loading) {
    return <Spin tip="Loading..." />;
  }

  return (
    <>
      <Descriptions title="User Info" bordered items={items} />
      <Transfer
        dataSource={apiData}
        titles={["未授权的API", "已授权的API"]}
        targetKeys={targetKeys}
        selectedKeys={selectedKeys}
        onChange={onChange}
        onSelectChange={onSelectChange}
        render={(item) => `${item.title} - ${item.description}`}
        listStyle={{
          width: 300,
          height: 400,
        }}
        disabled={true} // 不可操作
      />
    </>
  );
};

export default App;