import { createRoot } from "react-dom/client";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import AppLayout from "./AppLayout.tsx";
import LoginPage from "./LoginPage.tsx";
import AgentQueryPage from "./AgentQueryPage.tsx";
import AgentDetailPage from "./AgentDetailPage.tsx";
import HostQueryPage from "./HostQueryPage.tsx";
import HostDetailPage from "./HostDetailPage.tsx";
import LlmTaskQueryPage from "./LlmTaskQueryPage.tsx";
import LlmTaskDetailPage from "./LlmTaskDetailPage.tsx";
import RoleQueryPage from "./RoleQueryPage.tsx";
import RoleDetailPage from "./RoleDetailPage.tsx";
import RoleModifyPage from "./RoleModifyPage.tsx";
import RoleCreatePage from "./RoleCreatePage.tsx";
import UserQueryPage from "./UserQueryPage.tsx";
import UserDetailPage from "./UserDetailPage.tsx";
import UserCreatePage from "./UserCreatePage.tsx";
import UserModifyPage from "./UserModifyPage.tsx";
import SystemPage from "./SystemPage.tsx";
import restful_api from "./utils/restful_api.ts";
import RequireAuth from "./utils/RequireAuth.tsx";

async function init() {
  try {
    const [titleRsp, iconRsp] = await Promise.all([
      restful_api.get<{ title: string }>("/api/system/title"),
      restful_api.get<{ base64_icon: string }>("/api/system/icon"),
    ]);

    // 设置页面标题
    if (titleRsp.data.title) {
      document.title = titleRsp.data.title;
    }

    // 设置 favicon
    if (iconRsp.data.base64_icon) {
      let link: HTMLLinkElement | null =
        document.querySelector("link[rel~='icon']");
      if (!link) {
        link = document.createElement("link");
        link.rel = "icon";
        document.head.appendChild(link);
      }
      link.href = `data:image/png;base64,${iconRsp.data.base64_icon}`;
    }
  } catch (err) {
    console.error("初始化失败", err);
  }
  createRoot(document.getElementById("root")!).render(
    <BrowserRouter>
      <Routes>
        <Route path="login" element={<LoginPage />} />
        <Route element={<RequireAuth />}>
          <Route path="/" element={<AppLayout />}>
            <Route index element={<Navigate to="/agents" replace />} />
            <Route path="agents" element={<AgentQueryPage />} />
            <Route path="agents/:id" element={<AgentDetailPage />} />
            <Route path="hosts" element={<HostQueryPage />} />
            <Route path="hosts/:id" element={<HostDetailPage />} />
            <Route path="llm_tasks" element={<LlmTaskQueryPage />} />
            <Route path="llm_tasks/:id" element={<LlmTaskDetailPage />} />
            <Route path="roles" element={<RoleQueryPage />} />
            <Route path="roles/create" element={<RoleCreatePage />} />
            <Route path="roles/modify/:id" element={<RoleModifyPage />} />
            <Route path="roles/:id" element={<RoleDetailPage />} />
            <Route path="users" element={<UserQueryPage />} />
            <Route path="users/create" element={<UserCreatePage />} />
            <Route path="users/modify/:id" element={<UserModifyPage />} />
            <Route path="users/:id" element={<UserDetailPage />} />
            <Route path="system" element={<SystemPage />} />
          </Route>
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

init();
