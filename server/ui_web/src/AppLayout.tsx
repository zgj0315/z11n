import React from "react";
import {
  UploadOutlined,
  UserOutlined,
  VideoCameraOutlined,
} from "@ant-design/icons";
import { Layout, Menu, theme, Button, Breadcrumb } from "antd";
import { Outlet, useNavigate, useLocation } from "react-router-dom";
import restful_api from "./RESTfulApi.tsx";

const { Header, Content, Footer, Sider } = Layout;

const App: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();

  const {
    token: { colorBgContainer, borderRadiusLG },
  } = theme.useToken();

  const menuItems = [
    {
      key: "/home",
      icon: <UserOutlined />,
      label: "首页",
    },
    {
      key: "/pdf_articles",
      icon: <UserOutlined />,
      label: "文章列表",
    },
    {
      key: "/articles",
      icon: <UserOutlined />,
      label: "文章管理",
    },
    {
      key: "/files",
      icon: <UploadOutlined />,
      label: "文件管理",
    },
  ];
  const isLoggedIn = !!localStorage.getItem("token");
  if (isLoggedIn) {
    menuItems.push({
      key: "/pdf_article_access_logs",
      icon: <VideoCameraOutlined />,
      label: "浏览记录",
    });
    menuItems.push({
      key: "/logs",
      icon: <VideoCameraOutlined />,
      label: "操作日志",
    });
  }
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
                window.location.reload();
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
            items={[{ title: "Home" }, { title: "List" }, { title: "Article" }]}
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
        Self Examination ©{new Date().getFullYear()} Created by Zhaogj
      </Footer>
    </Layout>
  );
};

export default App;
