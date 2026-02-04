import React, { useState } from "react";
import { redirect } from "react-router";
import type { Route } from "./+types/home";
import { checkMembersEmpty } from "../api";
import { TopBar } from "../components/TopBar";
import { Sidebar } from "../components/Sidebar";
import { MainContent } from "../components/MainContent";
import { UploadModal } from "../components/UploadModal";
import { Modal, ModalContent, ModalHeader, ModalBody, ModalFooter, Button } from "@heroui/react";
import { Image, Video, Share2, Radio, Layers, Heart, Clock, Trash2 } from "lucide-react";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "HomeDrive" },
    { name: "description", content: "Your personal cloud storage" },
  ];
}

export async function loader() {
  try {
    const response = await checkMembersEmpty();
    if (response.is_empty) {
      return redirect("/setup");
    }
    return null;
  } catch (error) {
    console.error("Failed to check members:", error);
    return null;
  }
}

// Mobile menu items
const mobileMenuItems = [
  { key: "gallery", label: "图库", icon: Image },
  { key: "videos", label: "视频", icon: Video },
  { key: "shared", label: "共享", icon: Share2 },
  { key: "live-photos", label: "实况", icon: Radio },
  { key: "gifs", label: "GIF", icon: Layers },
  { key: "photos", label: "照片", icon: Image },
  { key: "favorites", label: "收藏", icon: Heart },
  { key: "recent", label: "最近", icon: Clock },
  { key: "trash", label: "回收站", icon: Trash2 },
];

export default function Home() {
  const [selectedView, setSelectedView] = useState("gallery");
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isUploadOpen, setIsUploadOpen] = useState(false);

  return (
    <div className="min-h-screen bg-background">
      <TopBar
        onMenuClick={() => setIsMenuOpen(true)}
        onUploadClick={() => setIsUploadOpen(true)}
      />
      <Sidebar selectedKey={selectedView} onSelect={setSelectedView} />
      <MainContent viewType={selectedView} />

      {/* Mobile Menu Modal */}
      <Modal
        isOpen={isMenuOpen}
        onClose={() => setIsMenuOpen(false)}
        placement="bottom"
        className="md:hidden m-0 rounded-t-2xl"
        motionProps={{
          variants: {
            enter: { y: "100%", opacity: 1 },
            exit: { y: "100%", opacity: 0 },
          },
        }}
      >
        <ModalContent>
          <ModalHeader className="flex flex-col gap-1 pb-2">
            菜单
            <span className="text-xs font-normal text-default-500">{mobileMenuItems.length} 个选项</span>
          </ModalHeader>
          <ModalBody className="py-2">
            <div className="grid grid-cols-3 gap-3">
              {mobileMenuItems.map((item) => (
                <Button
                  key={item.key}
                  variant="light"
                  className={`flex-col h-16 gap-1 ${
                    selectedView === item.key ? "text-primary bg-primary/10" : "text-default-600"
                  }`}
                  onPress={() => {
                    setSelectedView(item.key);
                    setIsMenuOpen(false);
                  }}
                >
                  <item.icon className="w-5 h-5" />
                  <span className="text-xs">{item.label}</span>
                </Button>
              ))}
            </div>
          </ModalBody>
          <ModalFooter className="justify-center pt-2">
            <Button variant="light" onPress={() => setIsMenuOpen(false)} className="w-full">
              取消
            </Button>
          </ModalFooter>
        </ModalContent>
      </Modal>

      {/* Upload Modal */}
      <UploadModal isOpen={isUploadOpen} onClose={() => setIsUploadOpen(false)} />
    </div>
  );
}
