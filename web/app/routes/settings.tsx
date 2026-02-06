import React, { useState, useEffect } from "react";
import { Link, useNavigate } from "react-router";
import {
  Card,
  CardHeader,
  CardBody,
  CardFooter,
  Avatar,
  Chip,
  Progress,
  Tabs,
  Tab,
  Button,
  Switch,
  Select,
  SelectItem,
} from "@heroui/react";
import { Settings, Clock, Trash2 } from "lucide-react";
import { useMediaQuery } from "~/hooks/useMediaQuery";
import { useAuth } from "../auth-context";

// Edit icon SVG
const EditIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
  </svg>
);

// Plus icon SVG
const PlusIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <line x1="12" y1="5" x2="12" y2="19" />
    <line x1="5" y1="12" x2="19" y2="12" />
  </svg>
);

// Refresh icon SVG
const RefreshIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <polyline points="23,4 23,10 17,10" />
    <polyline points="1,20 1,14 7,14" />
    <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
  </svg>
);

// Server icon SVG
const ServerIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <rect x="2" y="2" width="20" height="8" rx="2" ry="2" />
    <rect x="2" y="14" width="20" height="8" rx="2" ry="2" />
    <line x1="6" y1="6" x2="6.01" y2="6" />
    <line x1="6" y1="18" x2="6.01" y2="18" />
  </svg>
);

// Check icon SVG
const CheckIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <polyline points="20,6 9,17 4,12" />
  </svg>
);

// X icon SVG
const XIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <line x1="18" y1="6" x2="6" y2="18" />
    <line x1="6" y1="6" x2="18" y2="18" />
  </svg>
);

// Users icon SVG
const UsersIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
    <circle cx="9" cy="7" r="4" />
    <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
    <path d="M16 3.13a4 4 0 0 1 0 7.75" />
  </svg>
);

// Activity icon SVG
const ActivityIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <polyline points="22,12 18,12 15,21 9,3 6,12 2,12" />
  </svg>
);

// HardDrive icon SVG
const HardDriveIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <circle cx="12" cy="12" r="10" />
    <line x1="22" y1="12" x2="14" y2="12" />
    <line x1="10" y1="12" x2="2" y2="12" />
    <line x1="12" y1="2" x2="12" y2="8" />
    <line x1="12" y1="16" x2="12" y2="22" />
  </svg>
);

// CPU icon SVG
const CpuIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <rect x="4" y="4" width="16" height="16" rx="2" ry="2" />
    <rect x="9" y="9" width="6" height="6" />
    <line x1="9" y1="1" x2="9" y2="4" />
    <line x1="15" y1="1" x2="15" y2="4" />
    <line x1="9" y1="20" x2="9" y2="23" />
    <line x1="15" y1="20" x2="15" y2="23" />
    <line x1="20" y1="9" x2="23" y2="9" />
    <line x1="20" y1="14" x2="23" y2="14" />
    <line x1="1" y1="9" x2="4" y2="9" />
    <line x1="1" y1="14" x2="4" y2="14" />
  </svg>
);

// Memory icon SVG
const MemoryIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M6 2v6m6-6v6m6-6v6M4 6h16M4 10h16M4 14h16M4 18h16" />
  </svg>
);

// Upload icon SVG
const UploadIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
    <polyline points="17,8 12,3 7,8" />
    <line x1="12" y1="3" x2="12" y2="15" />
  </svg>
);

// Download icon SVG
const DownloadIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
    <polyline points="7,10 12,15 17,10" />
    <line x1="12" y1="15" x2="12" y2="3" />
  </svg>
);

// Process icon SVG
const ProcessIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
  </svg>
);

// Sync icon SVG
const SyncIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <polyline points="23,4 23,10 17,10" />
    <polyline points="1,20 1,14 7,14" />
    <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
  </svg>
);

// File icon SVG
const FileIcon = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
    <polyline points="14,2 14,8 20,8" />
    <line x1="16" y1="13" x2="8" y2="13" />
    <line x1="16" y1="17" x2="8" y2="17" />
    <polyline points="10,9 9,9 8,9" />
  </svg>
);

interface User {
  id: string;
  name: string;
  email: string;
  role: "admin" | "member";
  storageUsed: number;
  storageTotal: number;
  lastActive: string;
  status: "online" | "offline" | "away";
}

interface Task {
  id: string;
  name: string;
  type: "upload" | "download" | "process" | "sync";
  status: "pending" | "processing" | "completed" | "failed";
  progress: number;
  createdAt: string;
  size?: string;
}

// Mock data
const serverStats = {
  status: "online" as const,
  uptime: "15天 4小时 32分钟",
  cpu: 45,
  memory: 62,
  disk: {
    used: 256,
    total: 1024,
    percentage: 25,
  },
  network: {
    upload: "125 MB/s",
    download: "340 MB/s",
  },
};

const users: User[] = [
  {
    id: "1",
    name: "管理员",
    email: "admin@homedrive.local",
    role: "admin",
    storageUsed: 15.2,
    storageTotal: 100,
    lastActive: "刚刚",
    status: "online",
  },
  {
    id: "2",
    name: "张三",
    email: "zhangsan@example.com",
    role: "member",
    storageUsed: 8.5,
    storageTotal: 50,
    lastActive: "5分钟前",
    status: "online",
  },
  {
    id: "3",
    name: "李四",
    email: "lisi@example.com",
    role: "member",
    storageUsed: 2.3,
    storageTotal: 20,
    lastActive: "2小时前",
    status: "away",
  },
  {
    id: "4",
    name: "王五",
    email: "wangwu@example.com",
    role: "member",
    storageUsed: 0,
    storageTotal: 10,
    lastActive: "3天前",
    status: "offline",
  },
];

const tasks: Task[] = [
  {
    id: "1",
    name: "批量上传照片",
    type: "upload",
    status: "processing",
    progress: 67,
    createdAt: "10分钟前",
    size: "1.2 GB",
  },
  {
    id: "2",
    name: "同步备份",
    type: "sync",
    status: "pending",
    progress: 0,
    createdAt: "刚刚",
  },
  {
    id: "3",
    name: "视频转码",
    type: "process",
    status: "completed",
    progress: 100,
    createdAt: "1小时前",
    size: "256 MB",
  },
  {
    id: "4",
    name: "下载归档",
    type: "download",
    status: "failed",
    progress: 45,
    createdAt: "2小时前",
    size: "512 MB",
  },
];

const settings = {
  maxFileSize: 1024,
  allowPublicSharing: true,
  autoBackup: true,
  backupFrequency: "daily",
  theme: "dark" as const,
  language: "zh-CN",
  notifications: true,
  twoFactorAuth: false,
};

// Mobile User Card Component
function MobileUserCard({ user }: { user: User }) {
  return (
    <Card className="mb-3">
      <CardBody>
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-3">
            <Avatar
              name={user.name}
              size="md"
              isBordered
              color={
                user.status === "online"
                  ? "success"
                  : user.status === "away"
                  ? "warning"
                  : "default"
              }
            />
            <div>
              <p className="font-medium">{user.name}</p>
              <p className="text-xs text-default-500">{user.email}</p>
            </div>
          </div>
          <Chip
            size="sm"
            color={user.role === "admin" ? "primary" : "default"}
            variant="flat"
          >
            {user.role === "admin" ? "管理员" : "成员"}
          </Chip>
        </div>
        <div className="flex items-center justify-between text-sm mb-2">
          <span className="text-default-500">存储</span>
          <span>{user.storageUsed} / {user.storageTotal} GB</span>
        </div>
        <Progress
          value={(user.storageUsed / user.storageTotal) * 100}
          size="sm"
          color="primary"
          className="mb-3"
        />
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {user.status === "online" && <CheckIcon className="w-4 h-4 text-success" />}
            {user.status === "away" && <Clock className="w-4 h-4 text-warning" />}
            {user.status === "offline" && <XIcon className="w-4 h-4 text-default-400" />}
            <span className="text-sm text-default-500">
              {user.status === "online" ? "在线" : user.status === "away" ? "离开" : "离线"}
            </span>
            <span className="text-sm text-default-400">· {user.lastActive}</span>
          </div>
          <div className="flex gap-1">
            <Button isIconOnly size="sm" variant="light">
              <EditIcon className="w-4 h-4" />
            </Button>
            <Button isIconOnly size="sm" variant="light" color="danger">
              <Trash2 className="w-4 h-4" />
            </Button>
          </div>
        </div>
      </CardBody>
    </Card>
  );
}

// Mobile Task Card Component
function MobileTaskCard({ task }: { task: Task }) {
  return (
    <Card className="mb-3">
      <CardBody>
        <div className="flex items-start justify-between mb-3">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-default-100">
              {task.type === "upload" && <UploadIcon className="w-5 h-5 text-primary" />}
              {task.type === "download" && <DownloadIcon className="w-5 h-5 text-success" />}
              {task.type === "process" && <ProcessIcon className="w-5 h-5 text-warning" />}
              {task.type === "sync" && <SyncIcon className="w-5 h-5 text-secondary" />}
            </div>
            <div>
              <p className="font-medium">{task.name}</p>
              <p className="text-xs text-default-500">{task.createdAt}</p>
            </div>
          </div>
          <Chip
            size="sm"
            color={
              task.status === "completed"
                ? "success"
                : task.status === "processing"
                ? "primary"
                : task.status === "pending"
                ? "warning"
                : "danger"
            }
            variant="flat"
          >
            {task.status === "completed" && "完成"}
            {task.status === "processing" && "处理中"}
            {task.status === "pending" && "等待中"}
            {task.status === "failed" && "失败"}
          </Chip>
        </div>
        <div className="flex items-center justify-between text-sm mb-2">
          <span className="text-default-500">进度</span>
          <span className="text-sm">{task.progress}%</span>
        </div>
        <Progress
          value={task.progress}
          size="sm"
          color={
            task.status === "completed"
              ? "success"
              : task.status === "processing"
              ? "primary"
              : task.status === "pending"
              ? "warning"
              : "danger"
          }
          className="mb-2"
        />
        <div className="flex items-center justify-between">
          <span className="text-sm text-default-500">{task.size || "-"}</span>
          {task.status !== "completed" && (
            <Button size="sm" color="danger" variant="flat">
              取消
            </Button>
          )}
        </div>
      </CardBody>
    </Card>
  );
}

export default function SettingsPage() {
  const [currentTab, setCurrentTab] = useState("overview");
  const isMobile = useMediaQuery("(max-width: 768px)");
  const { isAuthenticated } = useAuth();
  const navigate = useNavigate();

  // 客户端登录检查
  useEffect(() => {
    if (!isAuthenticated) {
      navigate("/login", { replace: true });
    }
  }, [isAuthenticated, navigate]);

  // 未登录时不显示内容（会被重定向）
  if (!isAuthenticated) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="flex flex-col items-center gap-4">
          <div className="w-12 h-12 border-4 border-primary border-t-transparent rounded-full animate-spin" />
          <p className="text-default-500">正在检查登录状态...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-default-50 p-4 md:p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-6 md:mb-8">
          <h1 className="text-xl md:text-2xl font-bold text-foreground">系统设置</h1>
          <p className="text-default-500 mt-1 text-sm">管理服务器状态、用户和系统配置</p>
        </div>

        <Tabs
          selectedKey={currentTab}
          onSelectionChange={(key) => setCurrentTab(key as string)}
          variant={isMobile ? "solid" : "underlined"}
          classNames={{
            tabList: isMobile ? "bg-default-100 p-1 rounded-lg" : "gap-6",
            cursor: isMobile ? "bg-primary" : "bg-primary",
            tab: isMobile ? "h-10 px-4" : "px-0 h-12",
          }}
        >
          <Tab
            key="overview"
            title={
              <div className="flex items-center gap-2">
                <ActivityIcon className="w-4 h-4" />
                <span className="hidden sm:inline">概览</span>
              </div>
            }
          >
            <div className="mt-4 md:mt-6 grid grid-cols-2 md:grid-cols-4 gap-3 md:gap-4">
              {/* Server Status Card */}
              <Card className="border-none shadow-md">
                <CardBody className="flex flex-row items-center gap-3 p-4">
                  <div className={`p-2 rounded-xl ${serverStats.status === 'online' ? 'bg-success/20' : 'bg-danger/20'}`}>
                    <ServerIcon className={`w-5 h-5 md:w-6 md:h-6 ${serverStats.status === 'online' ? 'text-success' : 'text-danger'}`} />
                  </div>
                  <div className="min-w-0">
                    <p className="text-xs text-default-500 truncate">服务器</p>
                    <p className={`text-sm md:text-lg font-bold ${serverStats.status === 'online' ? 'text-success' : 'text-danger'}`}>
                      {serverStats.status === 'online' ? '运行中' : '离线'}
                    </p>
                  </div>
                </CardBody>
              </Card>

              {/* CPU Card */}
              <Card className="border-none shadow-md">
                <CardBody className="flex flex-row items-center gap-3 p-4">
                  <div className="p-2 rounded-xl bg-primary/20">
                    <CpuIcon className="w-5 h-5 md:w-6 md:h-6 text-primary" />
                  </div>
                  <div className="min-w-0">
                    <p className="text-xs text-default-500 truncate">CPU</p>
                    <p className="text-sm md:text-lg font-bold">{serverStats.cpu}%</p>
                  </div>
                </CardBody>
              </Card>

              {/* Memory Card */}
              <Card className="border-none shadow-md">
                <CardBody className="flex flex-row items-center gap-3 p-4">
                  <div className="p-2 rounded-xl bg-warning/20">
                    <MemoryIcon className="w-5 h-5 md:w-6 md:h-6 text-warning" />
                  </div>
                  <div className="min-w-0">
                    <p className="text-xs text-default-500 truncate">内存</p>
                    <p className="text-sm md:text-lg font-bold">{serverStats.memory}%</p>
                  </div>
                </CardBody>
              </Card>

              {/* Disk Card */}
              <Card className="border-none shadow-md">
                <CardBody className="flex flex-row items-center gap-3 p-4">
                  <div className="p-2 rounded-xl bg-secondary/20">
                    <HardDriveIcon className="w-5 h-5 md:w-6 md:h-6 text-secondary" />
                  </div>
                  <div className="min-w-0">
                    <p className="text-xs text-default-500 truncate">存储</p>
                    <p className="text-sm md:text-lg font-bold">{serverStats.disk.percentage}%</p>
                  </div>
                </CardBody>
              </Card>
            </div>

            {/* Detailed Stats - Desktop Table / Mobile Cards */}
            <div className="mt-4 md:mt-6 grid grid-cols-1 lg:grid-cols-2 gap-4 md:gap-6">
              {/* System Info */}
              <Card className="border-none shadow-md">
                <CardHeader>
                  <h2 className="text-base md:text-lg font-semibold">系统信息</h2>
                </CardHeader>
                <CardBody className="space-y-3 md:space-y-4">
                  <div className="flex items-center justify-between">
                    <span className="text-default-500 text-sm">运行时间</span>
                    <span className="font-medium text-sm">{serverStats.uptime}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-default-500 text-sm">网络上传</span>
                    <span className="font-medium text-sm">{serverStats.network.upload}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-default-500 text-sm">网络下载</span>
                    <span className="font-medium text-sm">{serverStats.network.download}</span>
                  </div>
                  <div className="space-y-2">
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-default-500">存储空间</span>
                      <span>{serverStats.disk.used} / {serverStats.disk.total} GB</span>
                    </div>
                    <Progress
                      value={serverStats.disk.percentage}
                      color="secondary"
                      className="h-2"
                    />
                  </div>
                </CardBody>
              </Card>

              {/* Quick Actions */}
              <Card className="border-none shadow-md">
                <CardHeader>
                  <h2 className="text-base md:text-lg font-semibold">快速操作</h2>
                </CardHeader>
                <CardBody>
                  <div className="grid grid-cols-2 gap-2 md:gap-3">
                    <Button
                      color="primary"
                      variant="flat"
                      size="sm"
                      startContent={<RefreshIcon className="w-4 h-4" />}
                    >
                      重启
                    </Button>
                    <Button
                      color="default"
                      variant="flat"
                      size="sm"
                      startContent={<FileIcon className="w-4 h-4" />}
                    >
                      缓存
                    </Button>
                    <Button
                      color="default"
                      variant="flat"
                      size="sm"
                    >
                      日志
                    </Button>
                    <Button
                      color="danger"
                      variant="flat"
                      size="sm"
                      startContent={<Trash2 className="w-4 h-4" />}
                    >
                      清理
                    </Button>
                  </div>
                </CardBody>
              </Card>
            </div>
          </Tab>

          <Tab
            key="users"
            title={
              <div className="flex items-center gap-2">
                <UsersIcon className="w-4 h-4" />
                <span className="hidden sm:inline">用户</span>
              </div>
            }
          >
            <div className="mt-4 md:mt-6">
              <Card className="border-none shadow-md">
                <CardHeader className="flex flex-col sm:flex-row justify-between gap-3">
                  <h2 className="text-base md:text-lg font-semibold">用户列表</h2>
                  <Button
                    color="primary"
                    size="sm"
                    startContent={<PlusIcon className="w-4 h-4" />}
                  >
                    添加用户
                  </Button>
                </CardHeader>
                <CardBody className={isMobile ? "p-0" : undefined}>
                  {isMobile ? (
                    // Mobile: Card list
                    <div className="p-4">
                      {users.map((user) => (
                        <MobileUserCard key={user.id} user={user} />
                      ))}
                    </div>
                  ) : (
                    // Desktop: Table
                    <div className="overflow-x-auto">
                      <table className="w-full">
                        <thead>
                          <tr className="border-b border-divider">
                            <th className="text-left p-4 text-sm font-medium text-default-500">用户</th>
                            <th className="text-left p-4 text-sm font-medium text-default-500">角色</th>
                            <th className="text-left p-4 text-sm font-medium text-default-500">存储</th>
                            <th className="text-left p-4 text-sm font-medium text-default-500">最后活跃</th>
                            <th className="text-left p-4 text-sm font-medium text-default-500">状态</th>
                            <th className="text-left p-4 text-sm font-medium text-default-500">操作</th>
                          </tr>
                        </thead>
                        <tbody>
                          {users.map((user) => (
                            <tr key={user.id} className="border-b border-divider hover:bg-default-50">
                              <td className="p-4">
                                <div className="flex items-center gap-3">
                                  <Avatar
                                    name={user.name}
                                    size="sm"
                                    isBordered
                                    color={
                                      user.status === "online"
                                        ? "success"
                                        : user.status === "away"
                                        ? "warning"
                                        : "default"
                                    }
                                  />
                                  <div>
                                    <p className="font-medium">{user.name}</p>
                                    <p className="text-xs text-default-500">{user.email}</p>
                                  </div>
                                </div>
                              </td>
                              <td className="p-4">
                                <Chip
                                  size="sm"
                                  color={user.role === "admin" ? "primary" : "default"}
                                  variant="flat"
                                >
                                  {user.role === "admin" ? "管理员" : "成员"}
                                </Chip>
                              </td>
                              <td className="p-4">
                                <div className="flex flex-col gap-1">
                                  <span className="text-sm">
                                    {user.storageUsed} / {user.storageTotal} GB
                                  </span>
                                  <Progress
                                    value={(user.storageUsed / user.storageTotal) * 100}
                                    size="sm"
                                    color="primary"
                                    className="h-1 w-24"
                                  />
                                </div>
                              </td>
                              <td className="p-4 text-sm text-default-500">{user.lastActive}</td>
                              <td className="p-4">
                                <div className="flex items-center gap-2">
                                  {user.status === "online" && <CheckIcon className="w-4 h-4 text-success" />}
                                  {user.status === "away" && <Clock className="w-4 h-4 text-warning" />}
                                  {user.status === "offline" && <XIcon className="w-4 h-4 text-default-400" />}
                                  <span className="text-sm capitalize">
                                    {user.status === "online" ? "在线" : user.status === "away" ? "离开" : "离线"}
                                  </span>
                                </div>
                              </td>
                              <td className="p-4">
                                <div className="flex items-center gap-2">
                                  <Button isIconOnly size="sm" variant="light">
                                    <EditIcon className="w-4 h-4" />
                                  </Button>
                                  <Button isIconOnly size="sm" variant="light" color="danger">
                                    <Trash2 className="w-4 h-4" />
                                  </Button>
                                </div>
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  )}
                </CardBody>
              </Card>
            </div>
          </Tab>

          <Tab
            key="tasks"
            title={
              <div className="flex items-center gap-2">
                <Clock className="w-4 h-4" />
                <span className="hidden sm:inline">任务</span>
              </div>
            }
          >
            <div className="mt-4 md:mt-6">
              <Card className="border-none shadow-md">
                <CardHeader className="flex flex-col sm:flex-row justify-between gap-3">
                  <h2 className="text-base md:text-lg font-semibold">当前任务</h2>
                  <Button
                    color="default"
                    size="sm"
                    variant="flat"
                    startContent={<RefreshIcon className="w-4 h-4" />}
                  >
                    刷新
                  </Button>
                </CardHeader>
                <CardBody className={isMobile ? "p-0" : undefined}>
                  {isMobile ? (
                    // Mobile: Card list
                    <div className="p-4">
                      {tasks.map((task) => (
                        <MobileTaskCard key={task.id} task={task} />
                      ))}
                    </div>
                  ) : (
                    // Desktop: Table
                    <div className="overflow-x-auto">
                      <table className="w-full">
                        <thead>
                          <tr className="border-b border-divider">
                            <th className="text-left p-4 text-sm font-medium text-default-500">任务名称</th>
                            <th className="text-left p-4 text-sm font-medium text-default-500">类型</th>
                            <th className="text-left p-4 text-sm font-medium text-default-500">状态</th>
                            <th className="text-left p-4 text-sm font-medium text-default-500">进度</th>
                            <th className="text-left p-4 text-sm font-medium text-default-500">大小</th>
                            <th className="text-left p-4 text-sm font-medium text-default-500">创建时间</th>
                            <th className="text-left p-4 text-sm font-medium text-default-500">操作</th>
                          </tr>
                        </thead>
                        <tbody>
                          {tasks.map((task) => (
                            <tr key={task.id} className="border-b border-divider hover:bg-default-50">
                              <td className="p-4">
                                <div className="flex items-center gap-3">
                                  <div className="p-2 rounded-lg bg-default-100">
                                    {task.type === "upload" && <UploadIcon className="w-4 h-4 text-primary" />}
                                    {task.type === "download" && <DownloadIcon className="w-4 h-4 text-success" />}
                                    {task.type === "process" && <ProcessIcon className="w-4 h-4 text-warning" />}
                                    {task.type === "sync" && <SyncIcon className="w-4 h-4 text-secondary" />}
                                  </div>
                                  <span className="font-medium">{task.name}</span>
                                </div>
                              </td>
                              <td className="p-4">
                                <Chip size="sm" variant="flat" color="default">
                                  {task.type === "upload" && "上传"}
                                  {task.type === "download" && "下载"}
                                  {task.type === "process" && "处理"}
                                  {task.type === "sync" && "同步"}
                                </Chip>
                              </td>
                              <td className="p-4">
                                <Chip
                                  size="sm"
                                  color={
                                    task.status === "completed"
                                      ? "success"
                                      : task.status === "processing"
                                      ? "primary"
                                      : task.status === "pending"
                                      ? "warning"
                                      : "danger"
                                  }
                                  variant="flat"
                                >
                                  {task.status === "completed" && "完成"}
                                  {task.status === "processing" && "处理中"}
                                  {task.status === "pending" && "等待中"}
                                  {task.status === "failed" && "失败"}
                                </Chip>
                              </td>
                              <td className="p-4">
                                <div className="flex items-center gap-2 w-32">
                                  <Progress
                                    value={task.progress}
                                    size="sm"
                                    color={
                                      task.status === "completed"
                                        ? "success"
                                        : task.status === "processing"
                                        ? "primary"
                                        : task.status === "pending"
                                        ? "warning"
                                        : "danger"
                                    }
                                    className="flex-1"
                                  />
                                  <span className="text-xs">{task.progress}%</span>
                                </div>
                              </td>
                              <td className="p-4 text-sm text-default-500">{task.size || "-"}</td>
                              <td className="p-4 text-sm text-default-500">{task.createdAt}</td>
                              <td className="p-4">
                                {task.status !== "completed" && (
                                  <Button size="sm" color="danger" variant="flat">
                                    取消
                                  </Button>
                                )}
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  )}
                </CardBody>
              </Card>
            </div>
          </Tab>

          <Tab
            key="config"
            title={
              <div className="flex items-center gap-2">
                <Settings className="w-4 h-4" />
                <span className="hidden sm:inline">配置</span>
              </div>
            }
          >
            <div className="mt-4 md:mt-6 grid grid-cols-1 lg:grid-cols-2 gap-4 md:gap-6">
              {/* General Settings */}
              <Card className="border-none shadow-md">
                <CardHeader>
                  <h2 className="text-base md:text-lg font-semibold">常规设置</h2>
                </CardHeader>
                <CardBody className="space-y-4 md:space-y-6">
                  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                      <p className="font-medium text-sm md:text-base">最大文件大小</p>
                      <p className="text-xs md:text-sm text-default-500">单个文件的最大上传大小</p>
                    </div>
                    <Select
                      size="sm"
                      defaultSelectedKeys={[String(settings.maxFileSize)]}
                      className="sm:w-32"
                    >
                      <SelectItem key="256">256 MB</SelectItem>
                      <SelectItem key="512">512 MB</SelectItem>
                      <SelectItem key="1024">1 GB</SelectItem>
                      <SelectItem key="2048">2 GB</SelectItem>
                    </Select>
                  </div>

                  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                      <p className="font-medium text-sm md:text-base">界面主题</p>
                      <p className="text-xs md:text-sm text-default-500">选择界面颜色主题</p>
                    </div>
                    <Select
                      size="sm"
                      defaultSelectedKeys={[settings.theme]}
                      className="sm:w-32"
                    >
                      <SelectItem key="dark">深色</SelectItem>
                      <SelectItem key="light">浅色</SelectItem>
                      <SelectItem key="system">跟随系统</SelectItem>
                    </Select>
                  </div>

                  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                      <p className="font-medium text-sm md:text-base">语言</p>
                      <p className="text-xs md:text-sm text-default-500">界面显示语言</p>
                    </div>
                    <Select
                      size="sm"
                      defaultSelectedKeys={[settings.language]}
                      className="sm:w-32"
                    >
                      <SelectItem key="zh-CN">中文</SelectItem>
                      <SelectItem key="en">English</SelectItem>
                    </Select>
                  </div>
                </CardBody>
              </Card>

              {/* Storage Settings */}
              <Card className="border-none shadow-md">
                <CardHeader>
                  <h2 className="text-base md:text-lg font-semibold">存储设置</h2>
                </CardHeader>
                <CardBody className="space-y-4 md:space-y-6">
                  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                      <p className="font-medium text-sm md:text-base">允许公开分享</p>
                      <p className="text-xs md:text-sm text-default-500">允许生成分享链接</p>
                    </div>
                    <Switch defaultSelected={settings.allowPublicSharing} />
                  </div>

                  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                      <p className="font-medium text-sm md:text-base">自动备份</p>
                      <p className="text-xs md:text-sm text-default-500">自动备份文件到云端</p>
                    </div>
                    <Switch defaultSelected={settings.autoBackup} />
                  </div>

                  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                      <p className="font-medium text-sm md:text-base">备份频率</p>
                      <p className="text-xs md:text-sm text-default-500">自动备份的时间间隔</p>
                    </div>
                    <Select
                      size="sm"
                      defaultSelectedKeys={[settings.backupFrequency]}
                      className="sm:w-32"
                    >
                      <SelectItem key="hourly">每小时</SelectItem>
                      <SelectItem key="daily">每天</SelectItem>
                      <SelectItem key="weekly">每周</SelectItem>
                      <SelectItem key="monthly">每月</SelectItem>
                    </Select>
                  </div>
                </CardBody>
              </Card>

              {/* Security Settings */}
              <Card className="border-none shadow-md">
                <CardHeader>
                  <h2 className="text-base md:text-lg font-semibold">安全设置</h2>
                </CardHeader>
                <CardBody className="space-y-4 md:space-y-6">
                  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                      <p className="font-medium text-sm md:text-base">消息通知</p>
                      <p className="text-xs md:text-sm text-default-500">接收系统消息通知</p>
                    </div>
                    <Switch defaultSelected={settings.notifications} />
                  </div>

                  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                      <p className="font-medium text-sm md:text-base">两步验证</p>
                      <p className="text-xs md:text-sm text-default-500">登录时需要二次验证</p>
                    </div>
                    <Switch defaultSelected={settings.twoFactorAuth} />
                  </div>
                </CardBody>
              </Card>
            </div>

            <div className="mt-6 flex flex-col sm:flex-row justify-end gap-3">
              <Button variant="flat">恢复默认</Button>
              <Button color="primary">保存设置</Button>
            </div>
          </Tab>
        </Tabs>
      </div>
    </div>
  );
}
