import React, { useState, useEffect } from "react";
import { useParams } from "react-router-dom";
import { Descriptions, Spin } from "antd";
import type { DescriptionsProps } from "antd";
import restful_api from "./RESTfulApi.tsx";

function jsonToDescriptionsItems(obj: Record<string, unknown>) {
  return Object.entries(obj)
    .filter(([key]) => key !== "processes")
    .map(([key, value], index) => ({
      key: key + index,
      label: key.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase()),
      children:
        typeof value === "object"
          ? JSON.stringify(value, null, 2)
          : String(value),
    }));
}

const App: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [items, setItems] = useState<DescriptionsProps["items"]>([]);
  const [loading, setLoading] = useState(true);
  useEffect(() => {
    restful_api
      .get(`/api/users/${id}`)
      .then((res) => {
        setItems(jsonToDescriptionsItems(res.data));
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
  return <Descriptions title="User Info" bordered items={items} />;
};

export default App;
