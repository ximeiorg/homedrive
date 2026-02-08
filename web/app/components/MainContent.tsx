import React, { useState, useEffect } from "react";
import { Card, CardBody, Button, ButtonGroup, Select, SelectItem, Chip } from "@heroui/react";
import { Grid3X3, List, Plus, Upload, Filter } from "lucide-react";
import { getFileList } from "../api";
import { PhotoProvider, PhotoView } from "react-photo-view";
import { VideoPlayerModal } from "./VideoPlayerModal";

interface MediaItem {
  id: string;
  thumbnail: string;
  videoUrl?: string;
  type: "image" | "video";
  title: string;
  date: string;
  fileSize?: number;
  width?: number;
  height?: number;
}

// 视频预览组件 - 鼠标悬停时播放
function VideoThumbnail({ src, poster }: { src?: string; poster?: string }) {
  const videoRef = React.useRef<HTMLVideoElement>(null);
  const [isHovered, setIsHovered] = React.useState(false);

  React.useEffect(() => {
    const video = videoRef.current;
    if (!video) return;

    if (isHovered) {
      video.currentTime = 0;
      video.play().catch(() => {});
    } else {
      video.pause();
    }
  }, [isHovered]);

  return (
    <video
      ref={videoRef}
      src={src}
      poster={poster}
      className="w-full h-full object-cover transition-transform group-hover:scale-105"
      preload="metadata"
      playsInline
      muted
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    />
  );
}

interface MainContentProps {
  viewType: string;
}

export function MainContent({ viewType }: MainContentProps) {
  const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
  const [files, setFiles] = useState<MediaItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  // 视频播放器状态
  const [isVideoPlayerOpen, setIsVideoPlayerOpen] = useState(false);
  const [currentVideoUrl, setCurrentVideoUrl] = useState<string>("");
  const [currentVideoTitle, setCurrentVideoTitle] = useState<string>("");

  // 打开视频播放器
  const openVideoPlayer = (videoUrl: string, title: string) => {
    setCurrentVideoUrl(videoUrl);
    setCurrentVideoTitle(title);
    setIsVideoPlayerOpen(true);
  };

  // 关闭视频播放器
  const closeVideoPlayer = () => {
    setIsVideoPlayerOpen(false);
    setCurrentVideoUrl("");
    setCurrentVideoTitle("");
  };

  // 获取文件列表
  useEffect(() => {
    const fetchFiles = async () => {
      try {
        setLoading(true);
        const response = await getFileList();
        
        // 将 API 返回的文件转换为 MediaItem 格式
        const token = localStorage.getItem("token");
        const mediaItems: MediaItem[] = response.files.map((file) => ({
          id: String(file.id),
          thumbnail: file.url ? `${file.url}?token=${token}` : `https://picsum.photos/seed/${file.id}/400/400`,
          videoUrl: file.mime_type?.startsWith("video/") && file.url ? `${file.url}?token=${token}` : undefined,
          type: file.mime_type?.startsWith("video/") ? "video" : "image",
          title: file.file_name,
          date: new Date(file.created_at).toISOString().split("T")[0],
        }));
        
        setFiles(mediaItems);
        setError(null);
      } catch (err) {
        console.error("Failed to fetch files:", err);
        setError("Failed to load files");
      } finally {
        setLoading(false);
      }
    };

    fetchFiles();
  }, [viewType]);

  const getTitle = () => {
    const titles: Record<string, string> = {
      gallery: "图库",
      videos: "视频",
      "live-photos": "实况照片",
      gifs: "GIF",
      photos: "照片",
      shared: "共享",
      favorites: "收藏",
      recent: "最近",
      trash: "回收站",
    };
    return titles[viewType] || "媒体库";
  };

  // Group media items by date
  const groupMediaByDate = (items: MediaItem[]) => {
    const groups: Record<string, MediaItem[]> = {};
    items.forEach((item) => {
      if (!groups[item.date]) {
        groups[item.date] = [];
      }
      groups[item.date].push(item);
    });
    return Object.entries(groups).sort((a, b) => b[0].localeCompare(a[0]));
  };

  // 加载状态
  if (loading) {
    return (
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
        <div className="flex flex-col items-center justify-center h-full">
          <div className="w-12 h-12 border-4 border-primary border-t-transparent rounded-full animate-spin" />
          <p className="text-default-500 mt-4">加载中...</p>
        </div>
      </main>
    );
  }

  // 错误状态
  if (error) {
    return (
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
        <div className="flex flex-col items-center justify-center h-full">
          <p className="text-danger">{error}</p>
        </div>
      </main>
    );
  }

  return (
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
          <h1 className="text-xl font-bold text-foreground">{getTitle()}</h1>
          <p className="text-sm text-default-500 mt-1">{files.length} 个项目</p>
        </div>
        
        <div className="flex items-center gap-2">
          
          {/* Desktop: Full controls */}
          <div className="hidden md:flex items-center gap-2">
            <Button
              variant="flat"
              size="sm"
              startContent={<Filter className="w-4 h-4" />}
            >
              筛选
            </Button>
            
            <ButtonGroup variant="flat" size="sm">
              <Button
                isIconOnly
                aria-label="Grid view"
                className={viewMode === "grid" ? "bg-primary/20" : ""}
                onPress={() => setViewMode("grid")}
              >
                <Grid3X3 className="w-4 h-4" />
              </Button>
              <Button
                isIconOnly
                aria-label="List view"
                className={viewMode === "list" ? "bg-primary/20" : ""}
                onPress={() => setViewMode("list")}
              >
                <List className="w-4 h-4" />
              </Button>
            </ButtonGroup>
            <Button
              color="primary"
              size="sm"
              startContent={<Upload className="w-4 h-4" />}
            >
              选择
            </Button>
          </div>
        </div>
      </div>

      {/* Sort - Desktop only */}
      <div className="hidden md:flex items-center justify-end mb-4">
        <Select
          id="sort-select-desktop"
          label="排序方式"
          placeholder="选择"
          size="sm"
          className="w-40"
        >
          <SelectItem key="date-desc">日期 (新到旧)</SelectItem>
          <SelectItem key="date-asc">日期 (旧到新)</SelectItem>
          <SelectItem key="name-asc">名称 (A-Z)</SelectItem>
          <SelectItem key="name-desc">名称 (Z-A)</SelectItem>
        </Select>
      </div>

      {/* Empty State */}
      {files.length === 0 ? (
        <div className="flex flex-col items-center justify-center h-64 text-center">
          <div className="w-16 h-16 md:w-24 md:h-24 rounded-full bg-default-100 flex items-center justify-center mb-4">
            <Upload className="w-8 h-8 md:w-10 md:h-10 text-default-400" />
          </div>
          <h3 className="text-base md:text-lg font-medium mb-2">还没有媒体文件</h3>
          <p className="text-sm text-default-500 mb-4">上传你的第一张照片或视频</p>
          <Button color="primary" size="sm" startContent={<Plus className="w-4 h-4" />}>
            选择文件上传
          </Button>
        </div>
      ) : viewMode === "grid" ? (
        /* Grid View */
        <PhotoProvider>
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-2 md:gap-4">
            {files.map((item) => (
              <Card
                key={item.id}
                shadow="sm"
                className="aspect-square overflow-hidden group"
              >
                <CardBody className="p-0">
                  <div className="relative w-full h-full overflow-hidden">
                    {item.type === "video" ? (
                      <div 
                        className="w-full h-full cursor-pointer"
                        onClick={() => item.videoUrl && openVideoPlayer(item.videoUrl, item.title)}
                      >
                        <VideoThumbnail src={item.videoUrl} poster={item.thumbnail} />
                      </div>
                    ) : (
                      <PhotoView src={item.thumbnail}>
                        <img
                          src={item.thumbnail}
                          alt={item.title}
                          className="w-full h-full object-cover transition-transform group-hover:scale-105 cursor-pointer"
                          loading="lazy"
                        />
                      </PhotoView>
                    )}
                    {item.type === "video" && (
                      <div className="absolute top-1 right-1 md:top-2 md:right-2">
                        <Chip
                          size="sm"
                          color="default"
                          variant="flat"
                          className="bg-black/60 text-white text-xs"
                        >
                          视频
                        </Chip>
                      </div>
                    )}
                    <div className="absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors pointer-events-none" />
                  </div>
                </CardBody>
              </Card>
            ))}
          </div>
        </PhotoProvider>
      ) : (
        /* List View */
        <PhotoProvider>
          <div className="space-y-6">
            {groupMediaByDate(files).map(([date, items]) => (
              <div key={date}>
                {/* Date Header */}
                <div className="sticky top-0 z-10 bg-default-50/95 backdrop-blur-sm py-2 mb-3">
                  <h3 className="text-sm font-semibold text-default-600">{date}</h3>
                </div>
                
                {/* Media Grid for this date */}
                <div className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 gap-2">
                  {items.map((item) => (
                    <Card
                      key={item.id}
                      shadow="sm"
                      className="aspect-square overflow-hidden group"
                    >
                      <CardBody className="p-0">
                        <div className="relative w-full h-full overflow-hidden">
                          {item.type === "video" ? (
                            <div 
                              className="w-full h-full cursor-pointer"
                              onClick={() => item.videoUrl && openVideoPlayer(item.videoUrl, item.title)}
                            >
                              <VideoThumbnail src={item.videoUrl} poster={item.thumbnail} />
                            </div>
                          ) : (
                            <PhotoView src={item.thumbnail}>
                              <img
                                src={item.thumbnail}
                                alt={item.title}
                                className="w-full h-full object-cover transition-transform group-hover:scale-105 cursor-pointer"
                                loading="lazy"
                              />
                            </PhotoView>
                          )}
                          {item.type === "video" && (
                            <div className="absolute top-1 right-1">
                              <Chip
                                size="sm"
                                color="default"
                                variant="flat"
                                className="bg-black/60 text-white text-xs"
                              >
                                视频
                              </Chip>
                            </div>
                          )}
                          <div className="absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors pointer-events-none" />
                        </div>
                      </CardBody>
                    </Card>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </PhotoProvider>
      )}

      {/* 视频播放器模态框 */}
      <VideoPlayerModal
        isOpen={isVideoPlayerOpen}
        onClose={closeVideoPlayer}
        videoUrl={currentVideoUrl}
        title={currentVideoTitle}
      />
    </main>
  );
}

// Helper for cn
import { cn } from "@heroui/react";
