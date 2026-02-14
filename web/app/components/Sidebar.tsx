import React from "react";
import { Link, useLocation, useSearchParams } from "react-router";
import { Button, Modal, ModalContent, ModalHeader, ModalBody } from "@heroui/react";
import {
  Image,
  Video,
  Share2,
  Radio,
  Grid3X3,
  Heart,
  Clock,
  Trash2,
  Home,
  Layers,
  Settings,
} from "lucide-react";
import { useAuth } from "../auth-context";

interface SidebarProps {
  selectedKey?: string;
  onSelect?: (key: string) => void;
  isMenuOpen?: boolean;
  onMenuClose?: () => void;
}

// Helper function to generate URL with type parameter
const getItemUrl = (key: string, searchParams: URLSearchParams): string => {
  if (key === "albums") {
    return "/albums";
  }
  if (key === "sharing") {
    return "/sharing";
  }
  if (key === "gallery" || key === "favorites" || key === "recent" || key === "trash") {
    return "/";
  }
  // For type items (videos, live-photos, gifs, photos), add type parameter
  return `/?type=${key}`;
};

// Helper function to check if a nav item is active
const isItemActive = (key: string, location: ReturnType<typeof useLocation>, searchParams: URLSearchParams): boolean => {
  if (key === "albums") {
    return location.pathname === "/albums";
  }
  if (key === "sharing") {
    return location.pathname === "/sharing";
  }
  if (key === "gallery") {
    return location.pathname === "/" && !searchParams.get("type");
  }
  if (key === "favorites" || key === "recent" || key === "trash") {
    return location.pathname === "/" && searchParams.get("type") === key;
  }
  // For type items
  return location.pathname === "/" && searchParams.get("type") === key;
};

// All navigation items for mobile menu
export const mobileMenuItems = [
  { key: "gallery", label: "图库", icon: Home },
  { key: "albums", label: "相册", icon: Grid3X3 },
  { key: "sharing", label: "共享", icon: Share2 },
  { key: "videos", label: "视频", icon: Video },
  { key: "live-photos", label: "实况", icon: Radio },
  { key: "gifs", label: "GIF", icon: Layers },
  { key: "photos", label: "照片", icon: Image },
  { key: "favorites", label: "收藏", icon: Heart },
  { key: "recent", label: "最近", icon: Clock },
  { key: "trash", label: "回收站", icon: Trash2 },
];

const mainItems = [
  { key: "gallery", label: "图库", icon: Home },
  { key: "albums", label: "相册", icon: Grid3X3 },
  { key: "sharing", label: "共享", icon: Share2 },
];

const typeItems = [
  { key: "videos", label: "视频", icon: Video },
  { key: "live-photos", label: "实况", icon: Radio },
  { key: "gifs", label: "GIF", icon: Layers },
  { key: "photos", label: "照片", icon: Image },
];

const utilityItems = [
  { key: "favorites", label: "收藏", icon: Heart },
  { key: "recent", label: "最近", icon: Clock },
  { key: "trash", label: "回收站", icon: Trash2 },
];

// Mock server status and storage info
const serverStatus = {
  online: true,
  storage: {
    used: 256, // GB
    total: 1024, // GB
  },
};

// Desktop Sidebar Component
function DesktopSidebar({ selectedKey }: { selectedKey?: string }) {
  const location = useLocation();
  const [searchParams] = useSearchParams();
  const { isAdmin } = useAuth();

  const renderNavItem = (item: typeof mainItems[0]) => {
    const url = getItemUrl(item.key, searchParams);
    const active = isItemActive(item.key, location, searchParams);

    return (
      <Link
        key={item.key}
        to={url}
        className={`flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
          active
            ? "bg-primary/10 text-primary"
            : "hover:bg-default-100 text-foreground"
        }`}
      >
        <item.icon className="w-5 h-5" />
        <span className="text-sm">{item.label}</span>
      </Link>
    );
  };

  return (
    <aside className="fixed left-0 top-16 bottom-0 w-64 bg-background dark:bg-default-900 border-r border-divider overflow-y-auto p-4 hidden md:block">
      <nav className="space-y-6">
        <div>
          <h3 className="text-xs font-semibold text-default-400 uppercase tracking-wider mb-3 px-3">
            媒体库
          </h3>
          <div className="flex flex-col gap-1">
            {mainItems.map(renderNavItem)}
          </div>
        </div>

        <div>
          <h3 className="text-xs font-semibold text-default-400 uppercase tracking-wider mb-3 px-3">
            类型
          </h3>
          <div className="flex flex-col gap-1">
            {typeItems.map((item) => {
              const url = getItemUrl(item.key, searchParams);
              const active = location.pathname === "/" && searchParams.get("type") === item.key;

              return (
                <Link
                  key={item.key}
                  to={url}
                  className={`flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
                    active
                      ? "bg-primary/10 text-primary"
                      : "hover:bg-default-100 text-foreground"
                  }`}
                >
                  <item.icon className="w-5 h-5" />
                  <span className="text-sm">{item.label}</span>
                </Link>
              );
            })}
          </div>
        </div>

        <div>
          <h3 className="text-xs font-semibold text-default-400 uppercase tracking-wider mb-3 px-3">
            工具
          </h3>
          <div className="flex flex-col gap-1">
            {utilityItems.map((item) => {
              const url = getItemUrl(item.key, searchParams);
              const active = location.pathname === "/" && searchParams.get("type") === item.key;

              return (
                <Link
                  key={item.key}
                  to={url}
                  className={`flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
                    active
                      ? "bg-primary/10 text-primary"
                      : "hover:bg-default-100 text-foreground"
                  }`}
                >
                  <item.icon className="w-5 h-5" />
                  <span className="text-sm">{item.label}</span>
                </Link>
              );
            })}
          </div>
        </div>
      </nav>

      {/* Server Status & Storage Section */}
      <div className="absolute bottom-0 left-0 right-0 p-4 border-t border-divider bg-background dark:bg-default-900">
        {/* Server Status */}
        <div className="flex items-center gap-2 mb-3">
          <div className={`w-2 h-2 rounded-full ${serverStatus.online ? 'bg-success animate-pulse' : 'bg-danger'}`} />
          <span className="text-xs text-default-500">
            {serverStatus.online ? '服务器在线' : '服务器离线'}
          </span>
        </div>

        {/* Storage Usage */}
        <div className="mb-3">
          <div className="flex items-center justify-between text-xs text-default-500 mb-1">
            <span>存储空间</span>
            <span>{serverStatus.storage.used} / {serverStatus.storage.total} GB</span>
          </div>
          <div className="w-full h-1.5 bg-default-200 rounded-full overflow-hidden">
            <div
              className="h-full bg-primary rounded-full transition-all duration-300"
              style={{ width: `${(serverStatus.storage.used / serverStatus.storage.total) * 100}%` }}
            />
          </div>
        </div>

        {/* Settings Link - Only visible to admin users */}
        {isAdmin && (
          <Link
            to="/settings"
            className="flex items-center gap-2 px-2 py-1.5 rounded-lg hover:bg-default-100 text-default-600 transition-colors"
          >
            <Settings className="w-4 h-4" />
            <span className="text-sm">系统设置</span>
          </Link>
        )}
      </div>
    </aside>
  );
}

// Mobile Floating Bottom Navigation Component
function MobileBottomNav({ selectedKey }: { selectedKey?: string }) {
  const location = useLocation();
  const [searchParams] = useSearchParams();

  return (
    <nav className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 md:hidden">
      <div className="flex items-center gap-2 px-3 py-2 bg-default-900/60 backdrop-blur-xl rounded-full border border-white/10 shadow-xl dark:bg-default-900/60 light:bg-white/80 light:border-black/10">
        {mainItems.map((item) => {
          const url = getItemUrl(item.key, searchParams);
          const active = isItemActive(item.key, location, searchParams);

          return (
            <Link
              key={item.key}
              to={url}
              className={`w-12 h-12 rounded-full flex items-center justify-center transition-all ${
                active
                  ? "bg-white/20 text-white shadow-lg"
                  : "text-white/70 hover:text-white hover:bg-white/10 dark:text-white/70 dark:hover:text-white dark:hover:bg-white/10 light:text-black/70 light:hover:text-black light:hover:bg-black/10"
              }`}
              aria-label={item.label}
            >
              <item.icon className="w-5 h-5" />
            </Link>
          );
        })}
        {/* Menu button */}
        <Button
          isIconOnly
          variant="light"
          className="w-12 h-12 rounded-full dark:text-white/70 dark:hover:text-white dark:hover:bg-white/10 light:text-black/70 light:hover:text-black light:hover:bg-black/10"
          aria-label="菜单"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <line x1="3" y1="12" x2="21" y2="12"></line>
            <line x1="3" y1="6" x2="21" y2="6"></line>
            <line x1="3" y1="18" x2="21" y2="18"></line>
          </svg>
        </Button>
      </div>
    </nav>
  );
}

// Mobile Menu Modal Component
interface MobileMenuModalProps {
  isOpen: boolean;
  onClose: () => void;
  selectedKey?: string;
  onSelect?: (key: string) => void;
}

export function MobileMenuModal({ isOpen, onClose, selectedKey }: MobileMenuModalProps) {
  const location = useLocation();
  const [searchParams] = useSearchParams();

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      placement="top"
      className="md:hidden m-0"
      motionProps={{
        variants: {
          enter: { y: 0, opacity: 1 },
          exit: { y: "-100%", opacity: 0 },
        },
        initial: { y: "-100%", opacity: 0 },
      }}
    >
      <ModalContent className="m-0 max-w-none rounded-b-xl">
        <ModalHeader className="flex flex-col gap-1 pb-2">
          菜单
          <span className="text-xs font-normal text-default-500">{mobileMenuItems.length} 个选项</span>
        </ModalHeader>
        <ModalBody className="py-2">
          <div className="grid grid-cols-3 gap-3">
            {mobileMenuItems.map((item) => {
              const url = getItemUrl(item.key, searchParams);
              const active = isItemActive(item.key, location, searchParams);

              return (
                <Link
                  key={item.key}
                  to={url}
                  className={`flex-col h-16 gap-1 flex items-center justify-center rounded-lg transition-colors ${
                    active
                      ? "text-primary bg-primary/10"
                      : "text-default-600 hover:bg-default-100"
                  }`}
                  onClick={onClose}
                >
                  <item.icon className="w-5 h-5" />
                  <span className="text-xs">{item.label}</span>
                </Link>
              );
            })}
          </div>
        </ModalBody>
      </ModalContent>
    </Modal>
  );
}

export function Sidebar({ selectedKey, isMenuOpen, onMenuClose }: SidebarProps) {
  return (
    <>
      <DesktopSidebar selectedKey={selectedKey} />
      <MobileBottomNav selectedKey={selectedKey} />
      {isMenuOpen && onMenuClose && (
        <MobileMenuModal
          isOpen={isMenuOpen}
          onClose={onMenuClose}
          selectedKey={selectedKey}
        />
      )}
    </>
  );
}
