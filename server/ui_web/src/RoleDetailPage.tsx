import React, { useState, useEffect } from "react";
import { useParams } from "react-router-dom";
import { Descriptions, Spin } from "antd";
import type { DescriptionsProps } from "antd";
import restful_api from "./RESTfulApi.tsx";

const App: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [items, setItems] = useState<DescriptionsProps["items"]>([]);
  const [loading, setLoading] = useState(true);
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
        const apiItems: DescriptionsProps["items"] = data.apis.map(
          (api: unknown, index: number) => ({
            key: `api-${index}`,
            label: `${api.restful_api.method}`,
            children: `${api.restful_api.name} (${api.restful_api.path} ${api.is_owned})`,
          })
        );
        setItems([...baseItems, ...apiItems]);
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
  return <Descriptions title="Role Info" bordered items={items} />;
};

export default App;
