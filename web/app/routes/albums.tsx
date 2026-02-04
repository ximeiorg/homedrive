import React, { useState } from "react";
import { redirect } from "react-router";
import { checkMembersEmpty } from "../api";
import { TopBar } from "../components/TopBar";
import { Sidebar } from "../components/Sidebar";
import { MainContent } from "../components/MainContent";
import { UploadModal } from "../components/UploadModal";

export function meta() {
  return [
    { title: "相册 - HomeDrive" },
    { name: "description", content: "Your photo albums" },
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

export default function Albums() {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isUploadOpen, setIsUploadOpen] = useState(false);

  return (
    <div className="min-h-screen bg-background">
      <TopBar
        onMenuClick={() => setIsMenuOpen(true)}
        onUploadClick={() => setIsUploadOpen(true)}
      />
      <Sidebar 
        selectedKey="albums" 
        isMenuOpen={isMenuOpen}
        onMenuClose={() => setIsMenuOpen(false)}
      />
      <MainContent viewType="gallery" />

      {/* Upload Modal */}
      <UploadModal isOpen={isUploadOpen} onClose={() => setIsUploadOpen(false)} />
    </div>
  );
}
