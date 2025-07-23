import React, { useState, useEffect } from "react";
import { useParams } from "react-router-dom";
import { Badge, Descriptions, Spin } from "antd";
import type { DescriptionsProps } from "antd";
import restful_api from "./RESTfulApi.tsx";

function jsonToDescriptionsItems(obj: Record<string, any>) {
  return Object.entries(obj).map(([key, value], index) => ({
    key: key + index,
    label: key.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase()),
    children:
      typeof value === "object"
        ? JSON.stringify(value, null, 2)
        : String(value),
  }));
}

// const items: DescriptionsProps["items"] = [
//   {
//     key: "1",
//     label: "Product",
//     children: "Cloud Database",
//   },
//   {
//     key: "2",
//     label: "Billing Mode",
//     children: "Prepaid",
//   },
//   {
//     key: "3",
//     label: "Automatic Renewal",
//     children: "YES",
//   },
//   {
//     key: "4",
//     label: "Order time",
//     children: "2018-04-24 18:00:00",
//   },
//   {
//     key: "5",
//     label: "Usage Time",
//     children: "2019-04-24 18:00:00",
//     span: 2,
//   },
//   {
//     key: "6",
//     label: "Status",
//     children: <Badge status="processing" text="Running" />,
//     span: 3,
//   },
//   {
//     key: "7",
//     label: "Negotiated Amount",
//     children: "$80.00",
//   },
//   {
//     key: "8",
//     label: "Discount",
//     children: "$20.00",
//   },
//   {
//     key: "9",
//     label: "Official Receipts",
//     children: "$60.00",
//   },
//   {
//     key: "10",
//     label: "Config Info",
//     children: (
//       <>
//         Data disk type: MongoDB
//         <br />
//         Database version: 3.4
//         <br />
//         Package: dds.mongo.mid
//         <br />
//         Storage space: 10 GB
//         <br />
//         Replication factor: 3
//         <br />
//         Region: East China 1
//         <br />
//       </>
//     ),
//   },
// ];

const App: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [items, setItems] = useState<DescriptionsProps["items"]>([]);
  // const [data, setData] = useState<unknown>(null);
  // const [data, setData] = useState<Record<string, unknown> | null>(null);
  const [loading, setLoading] = useState(true);
  useEffect(() => {
    restful_api
      .get(`/api/hosts/${id}`)
      .then((res) => {
        // setData(res.data);
        setItems(jsonToDescriptionsItems(res.data.system));
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
  return <Descriptions title="System Info" bordered items={items} />;
};

export default App;
