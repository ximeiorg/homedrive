import React, { useState, useEffect } from "react";
import { useSearchParams, useNavigate } from "react-router";
import { checkMembersEmpty } from "../api";
import { useAuth } from "../auth-context";
import { TopBar } from "../components/TopBar";
import { Sidebar } from "../components/Sidebar";
import { MainContent } from "../components/MainContent";
import { UploadModal } from "../components/UploadModal";

export default function Home() {
  const [searchParams] = useSearchParams();
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isUploadOpen, setIsUploadOpen] = useState(false);
  const { isAuthenticated } = useAuth();
  const navigate = useNavigate();

  // 客户端检查是否需要设置
  useEffect(() => {
    const checkSetup = async () => {
      try {
        const response = await checkMembersEmpty();
        if (response.is_empty) {
          navigate("/setup", { replace: true });
        }
      } catch (error) {
        console.error("Failed to check members:", error);
      }
    };

    checkSetup();
  }, [navigate]);

  // 客户端登录检查
  useEffect(() => {
    if (!isAuthenticated) {
      navigate("/login", { replace: true });
    }
  }, [isAuthenticated, navigate]);

  // Get viewType from URL params or default to "gallery"
  const viewType = searchParams.get("type") || "gallery";
  // Get search query from URL params
  const searchQuery = searchParams.get("q") || "";

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
        selectedKey={viewType} 
        isMenuOpen={isMenuOpen}
        onMenuClose={() => setIsMenuOpen(false)}
      />
      <MainContent viewType={viewType} searchQuery={searchQuery} />

      {/* Upload Modal */}
      <UploadModal isOpen={isUploadOpen} onClose={() => setIsUploadOpen(false)} />
    </div>
  );
}
