import React, { useEffect, useState } from "react";
import { Form, Input, Button, Upload, message, Spin } from "antd";
import { UploadOutlined } from "@ant-design/icons";
import restful_api from "./utils/restful_api.ts";

const SystemPage: React.FC = () => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(true);
  const [submittingTitle, setSubmittingTitle] = useState(false);
  const [submittingLogo, setSubmittingLogo] = useState(false);
  const [submittingIcon, setSubmittingIcon] = useState(false);
  const [previewLogo, setPreviewLogo] = useState<string>(
    "/android-chrome-512x512.png"
  );
  const [previewIcon, setPreviewIcon] = useState<string>("/favicon.ico");

  useEffect(() => {
    const fetchSystemData = async () => {
      try {
        const [titleRes, logoRes, iconRes] = await Promise.all([
          restful_api.get<{ title: string }>("/api/system/title"),
          restful_api.get<{ base64_logo: string }>("/api/system/logo"),
          restful_api.get<{ base64_icon: string }>("/api/system/icon"),
        ]);

        // 先重置表单再设置值，保证回显
        form.resetFields();
        form.setFieldsValue({ title: titleRes.data.title || "" });

        if (logoRes.data.base64_logo) {
          setPreviewLogo(`data:image/png;base64,${logoRes.data.base64_logo}`);
        }

        if (iconRes.data.base64_icon) {
          setPreviewIcon(`data:image/png;base64,${iconRes.data.base64_icon}`);
        }
      } catch (err) {
        console.error("获取系统信息失败:", err);
        message.error("获取系统信息失败");
      } finally {
        setLoading(false);
      }
    };

    fetchSystemData();
  }, [form]);

  const submitTitle = async (values: { title: string }) => {
    setSubmittingTitle(true);
    try {
      await restful_api.post("/api/system/title", { title: values.title });
      message.success("标题保存成功");
    } catch (err) {
      console.error("标题保存失败:", err);
      message.error("标题保存失败");
    } finally {
      setSubmittingTitle(false);
    }
  };

  const submitLogo = async (file: File) => {
    setSubmittingLogo(true);
    try {
      const formData = new FormData();
      formData.append("logo", file);
      await restful_api.post("/api/system/logo", formData, {
        headers: { "Content-Type": "multipart/form-data" },
      });

      const reader = new FileReader();
      reader.onload = () => setPreviewLogo(reader.result as string);
      reader.readAsDataURL(file);

      message.success("Logo保存成功");
    } catch (err) {
      console.error("Logo保存失败:", err);
      message.error("Logo保存失败");
    } finally {
      setSubmittingLogo(false);
    }
  };

  const submitIcon = async (file: File) => {
    setSubmittingIcon(true);
    try {
      const formData = new FormData();
      formData.append("icon", file);
      await restful_api.post("/api/system/icon", formData, {
        headers: { "Content-Type": "multipart/form-data" },
      });

      const reader = new FileReader();
      reader.onload = () => setPreviewIcon(reader.result as string);
      reader.readAsDataURL(file);

      message.success("Icon保存成功");
    } catch (err) {
      console.error("Icon保存失败:", err);
      message.error("Icon保存失败");
    } finally {
      setSubmittingIcon(false);
    }
  };

  if (loading) {
    return (
      <div style={{ textAlign: "center", marginTop: 50 }}>
        <Spin tip="加载中..." />
      </div>
    );
  }

  return (
    <div style={{ maxWidth: 600, margin: "0 auto", marginTop: 24 }}>
      {/* 标题设置 */}
      <Form
        key={loading ? "loading" : "loaded"}
        form={form}
        layout="vertical"
        onFinish={submitTitle}
      >
        <Form.Item
          label="系统标题"
          name="title"
          rules={[{ required: true, message: "请输入系统标题" }]}
        >
          <Input placeholder="请输入系统标题" />
        </Form.Item>
        <Form.Item>
          <Button
            type="primary"
            htmlType="submit"
            loading={submittingTitle}
            block
          >
            保存标题
          </Button>
        </Form.Item>
      </Form>

      {/* Logo 设置 */}
      <div style={{ marginTop: 32 }}>
        <label style={{ display: "block", marginBottom: 8 }}>Logo</label>
        <img
          src={previewLogo}
          alt="Logo预览"
          style={{ display: "block", height: 60, marginBottom: 8 }}
        />
        <Upload
          beforeUpload={(file) => {
            submitLogo(file);
            return false;
          }}
          showUploadList={false}
          accept="image/*"
        >
          <Button icon={<UploadOutlined />} loading={submittingLogo}>
            上传Logo
          </Button>
        </Upload>
      </div>

      {/* Icon 设置 */}
      <div style={{ marginTop: 32 }}>
        <label style={{ display: "block", marginBottom: 8 }}>
          Favicon (Icon)
        </label>
        <img
          src={previewIcon}
          alt="Icon预览"
          style={{ display: "block", height: 40, marginBottom: 8 }}
        />
        <Upload
          beforeUpload={(file) => {
            submitIcon(file);
            return false;
          }}
          showUploadList={false}
          accept="image/*"
        >
          <Button icon={<UploadOutlined />} loading={submittingIcon}>
            上传Icon
          </Button>
        </Upload>
      </div>
    </div>
  );
};

export default SystemPage;
