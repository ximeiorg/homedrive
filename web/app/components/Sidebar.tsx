import React from "react";
import { Button, Listbox, ListboxItem } from "@heroui/react";
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
} from "lucide-react";

interface SidebarProps {
  selectedKey?: string;
  onSelect?: (key: string) => void;
}

const mainItems = [
  { key: "gallery", label: "图库", icon: Home },
  { key: "videos", label: "视频", icon: Video },
  { key: "shared", label: "共享", icon: Share2 },
];

const typeItems = [
  { key: "live-photos", label: "实况", icon: Radio },
  { key: "gifs", label: "GIF", icon: Layers },
  { key: "photos", label: "照片", icon: Image },
];

const utilityItems = [
  { key: "favorites", label: "收藏", icon: Heart },
  { key: "recent", label: "最近", icon: Clock },
  { key: "trash", label: "回收站", icon: Trash2 },
];

// Desktop Sidebar Component
function DesktopSidebar({ selectedKey, onSelect }: { selectedKey?: string; onSelect?: (key: string) => void }) {
  return (
    <aside className="fixed left-0 top-16 bottom-0 w-64 bg-background border-r border-divider overflow-y-auto p-4 hidden md:block">
      <nav className="space-y-6">
        <div>
          <h3 className="text-xs font-semibold text-default-400 uppercase tracking-wider mb-3 px-3">
            媒体库
          </h3>
          <Listbox
            aria-label="Main navigation"
            items={mainItems}
            selectedKeys={selectedKey ? [selectedKey] : []}
            onSelectionChange={(keys) => {
              const key = Array.from(keys)[0] as string;
              onSelect?.(key);
            }}
            itemClasses={{
              base: [
                "data-[selected=true]:bg-primary/10",
                "data-[selected=true]:text-primary",
                "hover:bg-default-100",
                "transition-colors",
                "rounded-lg",
              ],
            }}
          >
            {(item) => (
              <ListboxItem
                key={item.key}
                startContent={<item.icon className="w-5 h-5" />}
                className="px-3 py-2"
              >
                {item.label}
              </ListboxItem>
            )}
          </Listbox>
        </div>

        <div>
          <h3 className="text-xs font-semibold text-default-400 uppercase tracking-wider mb-3 px-3">
            类型
          </h3>
          <Listbox
            aria-label="Media type navigation"
            items={typeItems}
            selectedKeys={selectedKey ? [selectedKey] : []}
            onSelectionChange={(keys) => {
              const key = Array.from(keys)[0] as string;
              onSelect?.(key);
            }}
            itemClasses={{
              base: [
                "data-[selected=true]:bg-primary/10",
                "data-[selected=true]:text-primary",
                "hover:bg-default-100",
                "transition-colors",
                "rounded-lg",
              ],
            }}
          >
            {(item) => (
              <ListboxItem
                key={item.key}
                startContent={<item.icon className="w-5 h-5" />}
                className="px-3 py-2"
              >
                {item.label}
              </ListboxItem>
            )}
          </Listbox>
        </div>

        <div>
          <h3 className="text-xs font-semibold text-default-400 uppercase tracking-wider mb-3 px-3">
            工具
          </h3>
          <Listbox
            aria-label="Utility navigation"
            items={utilityItems}
            selectedKeys={selectedKey ? [selectedKey] : []}
            onSelectionChange={(keys) => {
              const key = Array.from(keys)[0] as string;
              onSelect?.(key);
            }}
            itemClasses={{
              base: [
                "data-[selected=true]:bg-primary/10",
                "data-[selected=true]:text-primary",
                "hover:bg-default-100",
                "transition-colors",
                "rounded-lg",
              ],
            }}
          >
            {(item) => (
              <ListboxItem
                key={item.key}
                startContent={<item.icon className="w-5 h-5" />}
                className="px-3 py-2"
              >
                {item.label}
              </ListboxItem>
            )}
          </Listbox>
        </div>
      </nav>
    </aside>
  );
}

// Mobile Floating Bottom Navigation Component
function MobileBottomNav({ selectedKey, onSelect }: { selectedKey?: string; onSelect?: (key: string) => void }) {
  return (
    <nav className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 md:hidden">
      <div className="flex items-center gap-2 px-3 py-2 bg-default-900/60 backdrop-blur-xl rounded-full border border-white/10 shadow-xl">
        {mainItems.map((item) => (
          <Button
            key={item.key}
            variant="light"
            isIconOnly
            className={`w-12 h-12 rounded-full transition-all ${
              selectedKey === item.key
                ? "bg-white/20 text-white shadow-lg"
                : "text-white/70 hover:text-white hover:bg-white/10"
            }`}
            onPress={() => onSelect?.(item.key)}
            aria-label={item.label}
          >
            <item.icon className="w-5 h-5" />
          </Button>
        ))}
      </div>
    </nav>
  );
}

export function Sidebar({ selectedKey, onSelect }: SidebarProps) {
  return (
    <>
      <DesktopSidebar selectedKey={selectedKey} onSelect={onSelect} />
      <MobileBottomNav selectedKey={selectedKey} onSelect={onSelect} />
    </>
  );
}
