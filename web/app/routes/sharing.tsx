import React, { useState, useEffect } from "react";
import { useNavigate } from "react-router";
import { Card, CardBody, CardHeader, Avatar, Button, Chip } from "@heroui/react";
import { cn } from "@heroui/react";
import { TopBar } from "../components/TopBar";
import { Sidebar } from "../components/Sidebar";
import { UploadModal } from "../components/UploadModal";
import { User, Grid3X3 } from "lucide-react";
import { useAuth } from "../auth-context";

export function meta() {
  return [
    { title: "共享 - HomeDrive" },
    { name: "description", content: "Shared with you" },
  ];
}

// Demo data
const demoMembers = [
  {
    id: "member-1",
    name: "张三",
    email: "zhangsan@example.com",
    avatar: "https://i.pravatar.cc/150?u=member-1",
    sharedCount: 5,
  },
  {
    id: "member-2",
    name: "李四",
    email: "lisi@example.com",
    avatar: "https://i.pravatar.cc/150?u=member-2",
    sharedCount: 3,
  },
  {
    id: "member-3",
    name: "王五",
    email: "wangwu@example.com",
    avatar: "https://i.pravatar.cc/150?u=member-3",
    sharedCount: 8,
  },
];

const demoSharedAlbums = [
  {
    id: "album-1",
    name: "2024年假期照片",
    owner: "张三",
    cover: "https://picsum.photos/seed/album1/400/400",
    photoCount: 45,
    sharedAt: "2024-01-15",
  },
  {
    id: "album-2",
    name: "工作项目截图",
    owner: "李四",
    cover: "https://picsum.photos/seed/album2/400/400",
    photoCount: 12,
    sharedAt: "2024-01-10",
  },
  {
    id: "album-3",
    name: "家庭聚会",
    owner: "王五",
    cover: "https://picsum.photos/seed/album3/400/400",
    photoCount: 28,
    sharedAt: "2024-01-05",
  },
];

export default function Sharing() {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isUploadOpen, setIsUploadOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<"members" | "albums">("albums");
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
    <div className="min-h-screen bg-background">
      <TopBar
        onMenuClick={() => setIsMenuOpen(true)}
        onUploadClick={() => setIsUploadOpen(true)}
      />
      <Sidebar 
        selectedKey="sharing" 
        isMenuOpen={isMenuOpen}
        onMenuClose={() => setIsMenuOpen(false)}
      />

      <main
        className={cn(
          "overflow-y-auto bg-default-50 transition-all duration-300",
          "fixed left-0 right-0",
          "md:left-64",
          "top-16 bottom-0 md:bottom-0",
          "p-4 md:p-6",
          "pb-24 md:pb-6"
        )}
      >
        {/* Header */}
        <div className="flex items-center justify-between mb-4">
          <div>
            <h1 className="text-xl font-bold text-foreground">共享</h1>
            <p className="text-sm text-default-500 mt-1">
              {demoMembers.length} 位成员 • {demoSharedAlbums.length} 个共享相册
            </p>
          </div>
        </div>

        {/* Tabs */}
        <div className="flex gap-2 mb-4">
          <Button
            variant={activeTab === "albums" ? "solid" : "flat"}
            color="primary"
            size="sm"
            onPress={() => setActiveTab("albums")}
            startContent={<Grid3X3 className="w-4 h-4" />}
          >
            共享相册
          </Button>
          <Button
            variant={activeTab === "members" ? "solid" : "flat"}
            color="primary"
            size="sm"
            onPress={() => setActiveTab("members")}
            startContent={<User className="w-4 h-4" />}
          >
            成员
          </Button>
        </div>

        {/* Shared Albums Grid */}
        {activeTab === "albums" && (
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-2 md:gap-4">
            {demoSharedAlbums.map((album) => (
              <Card
                key={album.id}
                isPressable
                shadow="sm"
                className="aspect-square overflow-hidden group"
              >
                <CardBody className="p-0">
                  <div className="relative w-full h-full overflow-hidden">
                    <img
                      src={album.cover}
                      alt={album.name}
                      className="w-full h-full object-cover transition-transform group-hover:scale-105"
                    />
                    <div className="absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors" />
                    <div className="absolute bottom-0 left-0 right-0 p-2 bg-gradient-to-t from-black/60 to-transparent">
                      <p className="text-white text-sm font-medium truncate">{album.name}</p>
                      <p className="text-white/70 text-xs">来自 {album.owner}</p>
                    </div>
                  </div>
                </CardBody>
              </Card>
            ))}
          </div>
        )}

        {/* Members List */}
        {activeTab === "members" && (
          <div className="space-y-2">
            {demoMembers.map((member) => (
              <Card key={member.id} shadow="sm">
                <CardBody className="flex flex-row items-center gap-4 p-4">
                  <Avatar
                    src={member.avatar}
                    name={member.name}
                    className="w-12 h-12"
                  />
                  <div className="flex-1">
                    <p className="font-medium">{member.name}</p>
                    <p className="text-sm text-default-500">{member.email}</p>
                  </div>
                  <Chip size="sm" variant="flat">
                    {member.sharedCount} 个共享
                  </Chip>
                  <Button size="sm" variant="light" color="primary">
                    查看
                  </Button>
                </CardBody>
              </Card>
            ))}
          </div>
        )}

        {/* Empty State */}
        {demoSharedAlbums.length === 0 && activeTab === "albums" && (
          <div className="flex flex-col items-center justify-center h-64 text-center">
            <div className="w-16 h-16 md:w-24 md:h-24 rounded-full bg-default-100 flex items-center justify-center mb-4">
              <Grid3X3 className="w-8 h-8 md:w-10 md:h-10 text-default-400" />
            </div>
            <h3 className="text-base md:text-lg font-medium mb-2">还没有共享内容</h3>
            <p className="text-sm text-default-500">其他成员共享的相册会显示在这里</p>
          </div>
        )}
      </main>

      {/* Upload Modal */}
      <UploadModal isOpen={isUploadOpen} onClose={() => setIsUploadOpen(false)} />
    </div>
  );
}
