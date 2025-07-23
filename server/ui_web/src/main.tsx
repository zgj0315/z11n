import { createRoot } from "react-dom/client";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import AppLayout from "./AppLayout.tsx";
import LoginPage from "./LoginPage.tsx";
import AgentQueryPage from "./AgentQueryPage.tsx";
import AgentDetailPage from "./AgentDetailPage.tsx";
import HostQueryPage from "./HostQueryPage.tsx";

createRoot(document.getElementById("root")!).render(
  <BrowserRouter>
    <Routes>
      <Route path="login" element={<LoginPage />} />
      <Route path="/" element={<AppLayout />}>
        <Route index element={<Navigate to="/agents" replace />} />
        <Route path="agents" element={<AgentQueryPage />} />
        <Route path="agents/:id" element={<AgentDetailPage />} />
        <Route path="hosts" element={<HostQueryPage />} />
      </Route>
    </Routes>
  </BrowserRouter>
);
