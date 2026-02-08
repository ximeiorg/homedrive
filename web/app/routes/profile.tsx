import React, { useState, useEffect } from "react";
import { useNavigate } from "react-router";
import {
  Card,
  CardBody,
  Avatar,
  Button,
  Input,
  Textarea,
  Progress,
  Tabs,
  Tab,
} from "@heroui/react";
import { User, Image, Check, Clock } from "lucide-react";
import { useAuth } from "../auth-context";
import { TopBar } from "../components/TopBar";

// 格式化字节为 GB
const formatBytes = (bytes: number) => {
  if (bytes >= 1073741824) {
    return `${(bytes / 1073741824).toFixed(2)} GB`;
  }
  if (bytes >= 1048576) {
    return `${(bytes / 1048576).toFixed(2)} MB`;
  }
  return `${(bytes / 1024).toFixed(2)} KB`;
};

// 格式化日期
const formatDate = (dateStr: string) => {
  try {
    const date = new Date(dateStr);
    return date.toLocaleString("zh-CN");
  } catch {
    return dateStr;
  }
};

export default function ProfilePage() {
  const { member, isAuthenticated } = useAuth();
  const navigate = useNavigate();
  
  const [isEditing, setIsEditing] = useState(false);
  const [username, setUsername] = useState("");
  const [description, setDescription] = useState("");
  const [avatarUrl, setAvatarUrl] = useState("");
  const [fileCount, setFileCount] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);
  const [oldPassword, setOldPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [isChangingPassword, setIsChangingPassword] = useState(false);
  const [passwordError, setPasswordError] = useState("");
  const [passwordSuccess, setPasswordSuccess] = useState(false);

  // 初始化用户信息
  useEffect(() => {
    if (member) {
      setUsername(member.username);
      setAvatarUrl(member.avatar || "");
    }
  }, [member]);

  // 模拟获取用户文件数量
  useEffect(() => {
    if (isAuthenticated) {
      // TODO: 替换为真实的 API 调用
      setFileCount(35); // 临时模拟数据
    }
  }, [isAuthenticated]);

  // 客户端登录检查
  useEffect(() => {
    if (!isAuthenticated) {
      navigate("/login", { replace: true });
    }
  }, [isAuthenticated, navigate]);

  const handleSaveProfile = async () => {
    setIsLoading(true);
    // TODO: 替换为真实的 API 调用
    await new Promise((resolve) => setTimeout(resolve, 1000));
    setIsLoading(false);
    setIsEditing(false);
  };

  const handleChangePassword = async () => {
    setPasswordError("");
    setPasswordSuccess(false);
    
    if (newPassword !== confirmPassword) {
      setPasswordError("新密码与确认密码不匹配");
      return;
    }
    
    if (newPassword.length < 6) {
      setPasswordError("新密码长度至少为6位");
      return;
    }
    
    setIsChangingPassword(true);
    // TODO: 替换为真实的 API 调用
    await new Promise((resolve) => setTimeout(resolve, 1000));
    setIsChangingPassword(false);
    setPasswordSuccess(true);
    setOldPassword("");
    setNewPassword("");
    setConfirmPassword("");
  };

  const handleAvatarChange = () => {
    // TODO: 实现头像上传
    console.log("Avatar change clicked");
  };

  // 计算存储使用比例
  const storageUsed = 5.2 * 1024 * 1024 * 1024; // 5.2 GB - 临时模拟
  const storageTotal = 10 * 1024 * 1024 * 1024; // 10 GB - 临时模拟
  const storagePercent = (storageUsed / storageTotal) * 100;

  if (!member) {
    return null;
  }

  return (
    <div className="min-h-screen bg-default-50">
      <TopBar />
      
      <main className="pt-20 px-4 md:px-8 pb-8 max-w-4xl mx-auto">
        <h1 className="text-2xl font-bold mb-6">个人资料</h1>

        <Tabs variant="underlined" className="mb-6">
          <Tab key="profile" title="个人资料">
            <Card>
              <CardBody className="p-6">
                <div className="flex flex-col md:flex-row gap-8">
                  {/* 头像区域 */}
                  <div className="flex flex-col items-center gap-4">
                    <div className="relative">
                      <Avatar
                        isBordered
                        color="primary"
                        src={avatarUrl || undefined}
                        name={username.charAt(0).toUpperCase()}
                        className="w-24 h-24 text-3xl"
                      />
                      <Button
                        isIconOnly
                        size="sm"
                        color="primary"
                        variant="flat"
                        className="absolute bottom-0 right-0 rounded-full"
                        onPress={handleAvatarChange}
                      >
                        <Image className="w-4 h-4" />
                      </Button>
                    </div>
                    <Button
                      size="sm"
                      variant="flat"
                      onPress={handleAvatarChange}
                    >
                      更换头像
                    </Button>
                  </div>

                  {/* 用户信息表单 */}
                  <div className="flex-1 space-y-4">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <Input
                        label="用户名"
                        value={username}
                        onValueChange={setUsername}
                        isDisabled={!isEditing}
                        startContent={<User className="w-4 h-4 text-default-400" />}
                      />
                      <Input
                        label="存储标签"
                        value={member.storage_tag}
                        isDisabled
                        description="用于本地文件同步的标识"
                      />
                    </div>

                    <Textarea
                      label="个人简介"
                      value={description}
                      onValueChange={setDescription}
                      isDisabled={!isEditing}
                      placeholder="介绍一下你自己..."
                      minRows={3}
                    />

                    <div className="flex justify-end gap-2 pt-4">
                      {isEditing ? (
                        <>
                          <Button
                            variant="flat"
                            onPress={() => setIsEditing(false)}
                          >
                            取消
                          </Button>
                          <Button
                            color="primary"
                            onPress={handleSaveProfile}
                            isLoading={isLoading}
                            startContent={<Check className="w-4 h-4" />}
                          >
                            保存
                          </Button>
                        </>
                      ) : (
                        <Button
                          color="primary"
                          onPress={() => setIsEditing(true)}
                        >
                          编辑资料
                        </Button>
                      )}
                    </div>
                  </div>
                </div>
              </CardBody>
            </Card>
          </Tab>

          <Tab key="stats" title="存储统计">
            <Card>
              <CardBody className="p-6">
                <h3 className="text-lg font-medium mb-4">存储空间</h3>
                
                <div className="mb-4">
                  <div className="flex justify-between mb-2">
                    <span className="text-default-500">已使用</span>
                    <span className="font-medium">
                      {formatBytes(storageUsed)} / {formatBytes(storageTotal)}
                    </span>
                  </div>
                  <Progress
                    value={storagePercent}
                    color="primary"
                    size="lg"
                    className="mb-2"
                  />
                  <p className="text-sm text-default-400">
                    {storagePercent.toFixed(1)}% 已使用
                  </p>
                </div>

                <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-6">
                  <Card className="p-4 text-center">
                    <p className="text-2xl font-bold text-primary">{fileCount}</p>
                    <p className="text-sm text-default-500">文件数</p>
                  </Card>
                  <Card className="p-4 text-center">
                    <p className="text-2xl font-bold text-success">0</p>
                    <p className="text-sm text-default-500">相册数</p>
                  </Card>
                  <Card className="p-4 text-center">
                    <p className="text-2xl font-bold text-warning">0</p>
                    <p className="text-sm text-default-500">分享数</p>
                  </Card>
                  <Card className="p-4 text-center">
                    <p className="text-2xl font-bold text-secondary">0</p>
                    <p className="text-sm text-default-500">收藏数</p>
                  </Card>
                </div>

                <div className="mt-6 pt-6 border-t border-divider">
                  <h4 className="font-medium mb-4 flex items-center gap-2">
                    <Clock className="w-4 h-4" />
                    账户信息
                  </h4>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                    <div className="flex justify-between">
                      <span className="text-default-500">创建时间</span>
                      <span>{formatDate(member.created_at)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-default-500">用户 ID</span>
                      <span>{member.id}</span>
                    </div>
                  </div>
                </div>
              </CardBody>
            </Card>
          </Tab>

          <Tab key="security" title="安全">
            <Card>
              <CardBody className="p-6">
                <h3 className="text-lg font-medium mb-4">修改密码</h3>
                {passwordError && (
                  <p className="text-danger text-sm mb-4">{passwordError}</p>
                )}
                {passwordSuccess && (
                  <p className="text-success text-sm mb-4">密码修改成功</p>
                )}
                <div className="space-y-4 max-w-md">
                  <Input
                    label="当前密码"
                    type="password"
                    value={oldPassword}
                    onValueChange={setOldPassword}
                  />
                  <Input
                    label="新密码"
                    type="password"
                    value={newPassword}
                    onValueChange={setNewPassword}
                    description="密码长度至少6位"
                  />
                  <Input
                    label="确认新密码"
                    type="password"
                    value={confirmPassword}
                    onValueChange={setConfirmPassword}
                  />
                </div>
                <div className="mt-6">
                  <Button color="primary" onPress={handleChangePassword} isLoading={isChangingPassword}>
                    确认修改
                  </Button>
                </div>

                <div className="mt-6 pt-6 border-t border-divider">
                  <h4 className="font-medium mb-4">登录历史</h4>
                  <div className="text-sm text-default-500">
                    <p>上次登录：刚刚</p>
                    <p>登录 IP：127.0.0.1</p>
                  </div>
                </div>
              </CardBody>
            </Card>
          </Tab>
        </Tabs>
      </main>
    </div>
  );
}
