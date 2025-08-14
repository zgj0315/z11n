import React from "react";
import { UserOutlined } from "@ant-design/icons";
import { Layout, Menu, theme, Button, Breadcrumb } from "antd";
import { Outlet, useNavigate, useLocation } from "react-router-dom";
import restful_api from "./RESTfulApi.tsx";

const { Header, Content, Footer, Sider } = Layout;
interface RestfulApi {
  method: string;
  path: string;
  name: string;
}

function hasPermission(method: string, path: string) {
  try {
    const restful_apis = JSON.parse(
      localStorage.getItem("restful_apis") || "[]"
    );
    return restful_apis.some(
      (restful_api: RestfulApi) =>
        restful_api.method.toUpperCase() === method.toUpperCase() &&
        restful_api.path === path
    );
  } catch {
    return false;
  }
}
const App: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();

  const {
    token: { colorBgContainer, borderRadiusLG },
  } = theme.useToken();

  const menuItems = [
    hasPermission("GET", "/api/agents") && {
      key: "/agents",
      icon: <UserOutlined />,
      label: "Agent管理",
    },
    hasPermission("GET", "/api/hosts") && {
      key: "/hosts",
      icon: <UserOutlined />,
      label: "主机管理",
    },
    hasPermission("GET", "/api/llm_tasks") && {
      key: "/llm_tasks",
      icon: <UserOutlined />,
      label: "任务管理",
    },
    hasPermission("GET", "/api/roles") && {
      key: "/roles",
      icon: <UserOutlined />,
      label: "角色管理",
    },
    hasPermission("GET", "/api/users") && {
      key: "/users",
      icon: <UserOutlined />,
      label: "用户管理",
    },
  ];
  return (
    <Layout>
      <Header style={{ display: "flex", alignItems: "center" }}>
        <div className="demo-logo" />
        {localStorage.getItem("token") && (
          <Button
            type="link"
            onClick={async () => {
              const token = localStorage.getItem("token");
              if (token) {
                try {
                  await restful_api.post(`/api/logout/${token}`, null, {
                    headers: {
                      Authorization: `Bearer ${token}`,
                    },
                  });
                } catch (error) {
                  console.error("Logout failed", error);
                }
                localStorage.removeItem("token");
                navigate("/login", { replace: true });
              }
            }}
          >
            Logout
          </Button>
        )}
      </Header>
      <Layout>
        <Sider width={200} style={{ background: colorBgContainer }}>
          <Menu
            theme="dark"
            mode="inline"
            style={{ height: "100%", borderRight: 0 }}
            defaultSelectedKeys={["4"]}
            selectedKeys={[location.pathname]}
            onClick={({ key }) => navigate(key)}
            items={menuItems}
          />
        </Sider>
        <Layout style={{ padding: "0 24px 24px" }}>
          <Breadcrumb
            items={[{ title: "Home" }, { title: "Agent" }]}
            style={{ margin: "16px 0" }}
          />
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
