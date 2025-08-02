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

createRoot(document.getElementById("root")!).render(
  <BrowserRouter>
    <Routes>
      <Route path="login" element={<LoginPage />} />
      <Route path="/" element={<AppLayout />}>
        <Route index element={<Navigate to="/agents" replace />} />
        <Route path="agents" element={<AgentQueryPage />} />
        <Route path="agents/:id" element={<AgentDetailPage />} />
        <Route path="hosts" element={<HostQueryPage />} />
        <Route path="hosts/:id" element={<HostDetailPage />} />
        <Route path="llm_tasks" element={<LlmTaskQueryPage />} />
        <Route path="llm_tasks/:id" element={<LlmTaskDetailPage />} />
      </Route>
    </Routes>
  </BrowserRouter>
);
