import React, { useEffect, useMemo, useState } from "react";
import { UserOutlined } from "@ant-design/icons";
import { Layout, Menu, theme, Button, Breadcrumb, type MenuProps } from "antd";
import { Outlet, useNavigate, useLocation } from "react-router-dom";
import restful_api from "./utils/restful_api.ts";
import { hasPermission } from "./utils/permission";

const { Header, Content, Footer, Sider } = Layout;
// 菜单配置
const menuConfig = [
  {
    key: "/agents",
    icon: <UserOutlined />,
    label: "Agent管理",
    perm: ["GET", "/api/agents"],
  },
  {
    key: "/hosts",
    icon: <UserOutlined />,
    label: "主机管理",
    perm: ["GET", "/api/hosts"],
  },
  {
    key: "/llm_tasks",
    icon: <UserOutlined />,
    label: "任务管理",
    perm: ["GET", "/api/llm_tasks"],
  },
  {
    key: "/roles",
    icon: <UserOutlined />,
    label: "角色管理",
    perm: ["GET", "/api/roles"],
  },
  {
    key: "/users",
    icon: <UserOutlined />,
    label: "用户管理",
    perm: ["GET", "/api/users"],
  },
  {
    key: "/system",
    icon: <UserOutlined />,
    label: "系统设置",
    perm: ["GET", "/api/system"],
  },
];
const App: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const [username, setUsername] = useState<string>("");
  const {
    token: { colorBgContainer, borderRadiusLG },
  } = theme.useToken();
  const [logo, setLogo] = useState("/android-chrome-512x512.png");

  const menuItems: MenuProps["items"] = useMemo(
    () =>
      menuConfig
        .filter((item) => hasPermission(item.perm[0], item.perm[1]))
        .map(({ key, icon, label }) => ({
          key,
          icon,
          label,
        })),
    []
  );

  const breadcrumbItems = useMemo(() => {
    const paths = location.pathname.split("/").filter(Boolean);
    return [
      { title: "Home" },
      ...paths.map((p) => ({ title: p.charAt(0).toUpperCase() + p.slice(1) })),
    ];
  }, [location.pathname]);

  const handleLogout = async () => {
    const token = localStorage.getItem("token");
    localStorage.removeItem("token");
    localStorage.removeItem("restful_apis");
    navigate("/login", { replace: true });

    if (token) {
      try {
        await restful_api.post(`/api/logout/${token}`, null, {
          headers: { Authorization: `Bearer ${token}` },
        });
      } catch (error) {
        console.error("Logout failed", error);
      }
    }
  };
  useEffect(() => {
    const username = localStorage.getItem("username");
    if (username) setUsername(username);
    restful_api
      .get<{ base64_logo: string }>("/api/system/logo")
      .then((rsp) => {
        if (rsp.data.base64_logo) {
          setLogo(`data:image/png;base64,${rsp.data.base64_logo}`);
        }
      })
      .catch(console.error);
  }, []);
  return (
    <Layout>
      <Header
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "0 24px",
          background: "linear-gradient(90deg, #001529 0%, #002140 100%)",
          boxShadow: "0 2px 8px rgba(0, 0, 0, 0.15)",
        }}
      >
        {/* 左侧 Logo + 标题 */}
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <img src={logo} alt="Logo" style={{ height: 36, borderRadius: 6 }} />
          <span style={{ fontSize: 20, fontWeight: 600, color: "#fff" }}>
            管理系统
          </span>
        </div>

        {/* 右侧 用户名 + 退出按钮 */}
        {localStorage.getItem("token") && (
          <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <UserOutlined style={{ color: "#fff", fontSize: 18 }} />
              <span style={{ fontWeight: "bold", color: "#fff" }}>
                {username}
              </span>
            </div>
            <Button
              type="primary"
              danger
              size="small"
              onClick={handleLogout}
              style={{ borderRadius: 6 }}
            >
              Logout
            </Button>
          </div>
        )}
      </Header>
      <Layout>
        <Sider width={200} style={{ background: colorBgContainer }}>
          <Menu
            theme="dark"
            mode="inline"
            style={{ height: "100%", borderRight: 0 }}
            onClick={({ key }) => navigate(key)}
            items={menuItems}
          />
        </Sider>
        <Layout style={{ padding: "0 24px 24px" }}>
          <Breadcrumb items={breadcrumbItems} style={{ margin: "16px 0" }} />
          <Content
            style={{
              padding: 24,
              margin: 0,
              minHeight: 280,
              background: colorBgContainer,
              borderRadius: borderRadiusLG,
            }}
          >
            <div
              style={{
                padding: 24,
                minHeight: 360,
                background: colorBgContainer,
                borderRadius: borderRadiusLG,
              }}
            >
              <Outlet />
            </div>
          </Content>
        </Layout>
      </Layout>
      <Footer style={{ textAlign: "center" }}>
        Z11N ©{new Date().getFullYear()} Created by Zhaogj
      </Footer>
    </Layout>
  );
};

export default App;
