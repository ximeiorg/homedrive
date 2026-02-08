import React from "react";
import { Avatar, Input, Dropdown, DropdownTrigger, DropdownMenu, DropdownItem, Button } from "@heroui/react";
import { Search, Settings, LogOut, User, Menu, Upload } from "lucide-react";
import { useAuth } from "../auth-context";
import { useNavigate } from "react-router";

interface TopBarProps {
  onMenuClick?: () => void;
  onUploadClick?: () => void;
}

export function TopBar({ onMenuClick, onUploadClick }: TopBarProps) {
  const { member, logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = () => {
    logout();
    navigate("/login");
  };

  // 获取用户名首字母作为头像
  const getInitials = (name: string) => {
    return name.charAt(0).toUpperCase();
  };

  // 获取头像 URL（如果 member.avatar 为 null 则使用首字母）
  const avatarUrl = member?.avatar || undefined;
  const initials = member?.avatar ? undefined : getInitials(member?.username || "U");

  return (
    <header className="fixed top-0 left-0 right-0 h-16 bg-background/80 backdrop-blur-md border-b border-divider z-50 px-4 flex items-center justify-between">
      {/* Left: Logo & Menu Button */}
      <div className="flex items-center gap-4">
        <Button
          isIconOnly
          variant="light"
          className="md:hidden"
          onPress={onMenuClick}
          aria-label="Menu"
        >
          <Menu className="w-5 h-5" />
        </Button>
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 bg-gradient-to-br from-primary to-secondary rounded-lg flex items-center justify-center">
            <span className="text-white font-bold text-lg">H</span>
          </div>
          <span className="text-xl font-bold text-foreground hidden sm:block">HomeDrive</span>
        </div>
      </div>

      {/* Center: Search Bar - Hidden on mobile */}
      <div className="flex-1 max-w-2xl mx-4 hidden md:block">
        <Input
          placeholder="搜索照片、视频..."
          startContent={<Search className="text-default-400 w-4 h-4" />}
          size="lg"
          radius="full"
          classNames={{
            inputWrapper: "bg-default-100 hover:bg-default-200 transition-colors",
          }}
        />
      </div>

      {/* Right: Actions */}
      <div className="flex items-center gap-2">
        {/* Upload Button */}
        <Button
          variant="flat"
          color="primary"
          startContent={<Upload className="w-4 h-4" />}
          onPress={onUploadClick}
          className="hidden sm:flex"
        >
          上传
        </Button>
        
        {/* Mobile Upload Button */}
        <Button
          isIconOnly
          variant="flat"
          color="primary"
          onPress={onUploadClick}
          className="sm:hidden"
          aria-label="Upload"
        >
          <Upload className="w-5 h-5" />
        </Button>

        {/* User Profile */}
        <Dropdown placement="bottom-end">
          <DropdownTrigger>
            <Button variant="light" className="p-2 rounded-full">
              <Avatar
                isBordered
                color="primary"
                src={avatarUrl}
                name={initials}
                size="sm"
              />
            </Button>
          </DropdownTrigger>
          <DropdownMenu aria-label="User menu" variant="flat">
            <DropdownItem 
              key="profile" 
              startContent={<User className="w-4 h-4" />}
              onPress={() => navigate("/profile")}
            >
              个人资料
            </DropdownItem>
            <DropdownItem 
              key="settings" 
              startContent={<Settings className="w-4 h-4" />}
              onPress={() => navigate("/settings")}
            >
              系统设置
            </DropdownItem>
            <DropdownItem 
              key="logout" 
              color="danger" 
              startContent={<LogOut className="w-4 h-4" />}
              onPress={handleLogout}
            >
              退出登录
            </DropdownItem>
          </DropdownMenu>
        </Dropdown>
      </div>
    </header>
  );
}
