import React, { useState, useEffect } from "react";
import { useParams } from "react-router-dom";
import { Descriptions, Spin, Transfer, List, Tag, Typography } from "antd";
import type { DescriptionsProps, TransferProps } from "antd";
import restful_api from "./RESTfulApi.tsx";

interface ApiRecord {
  key: string;
  title: string;
  description: string;
}

interface Role {
  id: number;
  name: string;
  restful_apis: RestfulApi[];
}

interface RestfulApi {
  method: string;
  path: string;
  name: string;
}

const App: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [roles, setRoles] = useState<Role[]>([]);
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
    interface RestfulApi {
      method: string;
      path: string;
      name: string;
    }
    restful_api
      .get(`/api/restful_apis`)
      .then((res) => {
        const transferData: ApiRecord[] = res.data.map(
          (restful_api: RestfulApi) => ({
            key: `${restful_api.method}-${restful_api.path}`,
            title: `${restful_api.name}`,
            description: `${restful_api.method} ${restful_api.path}`,
          })
        );
        setApiData(transferData);
      })
      .catch((err) => {
        console.error("Failed to fetch system info:", err);
      })
      .finally(() => {});
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

        const apiMap = new Map<string, ApiRecord>();
        const ownedKeysSet = new Set<string>();
        setRoles(data.roles);
        console.log("roles: ", roles);
        data.roles.forEach((role) => {
          role.restful_apis.forEach((restful_api) => {
            const key = `${restful_api.method}-${restful_api.path}`;
            if (!apiMap.has(key)) {
              apiMap.set(key, {
                key,
                title: restful_api.name,
                description: `${restful_api.method} ${restful_api.path}`,
              });
              ownedKeysSet.add(key);
            }
          });
        });
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

      <Typography.Title level={5} style={{ marginTop: 16 }}>
        角色列表
      </Typography.Title>
      <List
        bordered
        dataSource={roles}
        renderItem={(role) => (
          <List.Item>
            <Tag color="blue">{role.name}</Tag>
          </List.Item>
        )}
        style={{ marginBottom: 24 }}
      />
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
