import React, { useState, useEffect } from "react";
import { useParams } from "react-router-dom";
import { Descriptions, Spin, Tabs, Table } from "antd";
import type { DescriptionsProps, TabsProps, TableProps } from "antd";
import restful_api from "./RESTfulApi.tsx";

interface ProcessType {
  pid: number;
  name: string;
  exe: string;
  status: string;
}

const columns: TableProps<ProcessType>["columns"] = [
  {
    title: "Pid",
    dataIndex: "pid",
    key: "pid",
  },
  {
    title: "Name",
    dataIndex: "name",
    key: "name",
  },
  {
    title: "Exe",
    dataIndex: "exe",
    key: "exe",
  },
  {
    title: "Status",
    dataIndex: "status",
    key: "status",
  },
];

const onChange = (key: string) => {
  console.log(key);
};

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
  const [systemItems, setSystemItems] = useState<DescriptionsProps["items"]>(
    []
  );
  const [processItems, setProcessItems] = useState<ProcessType[]>([]);
  const [loading, setLoading] = useState(true);
  const tabItems: TabsProps["items"] = [
    {
      key: "system_info",
      label: "System Info",
      children: (
        <Descriptions title="System Info" bordered items={systemItems} />
      ),
    },
    {
      key: "processes",
      label: "Processes",
      children: (
        <Table<ProcessType> columns={columns} dataSource={processItems} />
      ),
    },
  ];
  useEffect(() => {
    restful_api
      .get(`/api/hosts/${id}`)
      .then((res) => {
        setSystemItems(jsonToDescriptionsItems(res.data.system));
        setProcessItems(res.data.system.processes);
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

  if (!systemItems) {
    return <div>No data</div>;
  }
  return (
    <>
      <Tabs defaultActiveKey="1" items={tabItems} onChange={onChange} />
    </>
  );
};

export default App;
