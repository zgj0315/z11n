import React, { useState, useEffect } from "react";
import { useParams } from "react-router-dom";
import { Descriptions, Spin, Tabs, Table } from "antd";
import type { DescriptionsProps, TabsProps, TableProps } from "antd";
import restful_api from "./utils/restful_api.ts";

interface ProcessType {
  pid: number;
  name: string;
  exe: string;
  status: string;
}

interface DiskType {
  name: string;
  file_system: string;
  mount_point: string;
  kind: string;
  available_space: number;
  total_space: number;
  is_removable: boolean;
  is_read_only: boolean;
}

interface NetworkType {
  interface_name: string;
  total_received: number;
  total_transmitted: number;
  addrs: [string];
}

const process_columns: TableProps<ProcessType>["columns"] = [
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

const disk_columns: TableProps<DiskType>["columns"] = [
  {
    title: "Name",
    dataIndex: "name",
    key: "name",
  },
  {
    title: "File System",
    dataIndex: "file_system",
    key: "file_system",
  },
  {
    title: "Mount Point",
    dataIndex: "mount_point",
    key: "mount_point",
  },
  {
    title: "Kind",
    dataIndex: "kind",
    key: "kind",
  },
  {
    title: "Total Space",
    dataIndex: "total_space",
    key: "total_space",
  },
  {
    title: "Available Space",
    dataIndex: "available_space",
    key: "available_space",
  },
  {
    title: "Is Removable",
    dataIndex: "is_removable",
    key: "is_removable",
  },
  {
    title: "Is Read Only",
    dataIndex: "is_read_only",
    key: "is_read_only",
  },
];

const network_columns: TableProps<NetworkType>["columns"] = [
  {
    title: "Interface Name",
    dataIndex: "interface_name",
    key: "interface_name",
  },
  {
    title: "Total Received",
    dataIndex: "total_received",
    key: "total_received",
  },
  {
    title: "Total Transmitted",
    dataIndex: "total_transmitted",
    key: "total_transmitted",
  },
  {
    title: "Addrs",
    dataIndex: "addrs",
    key: "addrs",
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
  const [diskItems, setDiskItems] = useState<DiskType[]>([]);
  const [networkItems, setNetworkItems] = useState<NetworkType[]>([]);
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
      key: "process",
      label: "Process",
      children: (
        <Table<ProcessType>
          columns={process_columns}
          dataSource={processItems}
        />
      ),
    },
    {
      key: "disk",
      label: "Disk",
      children: (
        <Table<DiskType> columns={disk_columns} dataSource={diskItems} />
      ),
    },
    {
      key: "network",
      label: "Network",
      children: (
        <Table<NetworkType>
          columns={network_columns}
          dataSource={networkItems}
        />
      ),
    },
  ];
  useEffect(() => {
    restful_api
      .get(`/api/hosts/${id}`)
      .then((res) => {
        setSystemItems(jsonToDescriptionsItems(res.data.system));
        setProcessItems(res.data.system.processes);
        setDiskItems(res.data.disks);
        setNetworkItems(res.data.networks);
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
