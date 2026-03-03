import React, { useState, useEffect, useCallback, useRef } from "react";
import { Card, CardBody, Button, ButtonGroup, Select, SelectItem, Chip, Spinner, Modal, ModalContent, ModalHeader, ModalBody, ModalFooter, useDisclosure } from "@heroui/react";
import { Grid3X3, List, Plus, Upload, Filter, Trash2, X, Check } from "lucide-react";
import { getFileList, FILES_API, authFetch, deleteFiles, getTrashList, restoreFiles, emptyTrash } from "../api";
import type { DeleteFilesResponse } from "../api";
import { PhotoProvider, PhotoView } from "react-photo-view";
import { VideoPlayerModal } from "./VideoPlayerModal";
import InfiniteScroll from "react-infinite-scroll-component";

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
  searchQuery?: string;
}

export function MainContent({ viewType, searchQuery = "" }: MainContentProps) {
  const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
  const [files, setFiles] = useState<MediaItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  // 分页状态
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [hasMore, setHasMore] = useState(true);
  const PAGE_SIZE = 50;
  
  // 滚动容器引用
  const scrollRef = useRef<HTMLDivElement>(null);
  
  // 视频播放器状态
  const [isVideoPlayerOpen, setIsVideoPlayerOpen] = useState(false);
  const [currentVideoUrl, setCurrentVideoUrl] = useState<string>("");
  const [currentVideoTitle, setCurrentVideoTitle] = useState<string>("");

  // 多选模式状态
  const [isSelectMode, setIsSelectMode] = useState(false);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [isDeleting, setIsDeleting] = useState(false);

  // 删除确认对话框
  const { isOpen: isDeleteModalOpen, onOpen: onDeleteModalOpen, onClose: onDeleteModalClose } = useDisclosure();

  // 打开视频播放器
  const openVideoPlayer = (videoUrl: string, title: string) => {
    if (isSelectMode) return; // 选择模式下不打开播放器
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

  // 切换选择模式
  const toggleSelectMode = () => {
    setIsSelectMode(!isSelectMode);
    setSelectedIds(new Set());
  };

  // 切换单个文件选择
  const toggleFileSelection = (id: string) => {
    setSelectedIds((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(id)) {
        newSet.delete(id);
      } else {
        newSet.add(id);
      }
      return newSet;
    });
  };

  // 全选/取消全选
  const toggleSelectAll = () => {
    if (selectedIds.size === files.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(files.map((f) => f.id)));
    }
  };

  // 删除选中的文件
  const handleDeleteSelected = async () => {
    if (selectedIds.size === 0) return;
    
    setIsDeleting(true);
    try {
      const ids = Array.from(selectedIds).map((id) => parseInt(id, 10));
      
      // 如果是回收站视图，执行永久删除
      if (viewType === "trash") {
        const result = await emptyTrash();
        if (result.success) {
          setFiles((prev) => prev.filter((f) => !selectedIds.has(f.id)));
          setSelectedIds(new Set());
          setIsSelectMode(false);
        }
        console.log(result.message);
      } else {
        // 否则移动到回收站
        const result: DeleteFilesResponse = await deleteFiles(ids);
        
        if (result.success) {
          // 从列表中移除已删除的文件
          setFiles((prev) => prev.filter((f) => !selectedIds.has(f.id)));
          setSelectedIds(new Set());
          setIsSelectMode(false);
        }
        
        console.log(result.message);
      }
    } catch (err) {
      console.error("Failed to delete files:", err);
      setError("删除文件失败");
    } finally {
      setIsDeleting(false);
      onDeleteModalOpen();
    }
  };

  // 恢复选中的文件
  const handleRestoreSelected = async () => {
    if (selectedIds.size === 0) return;
    
    setIsDeleting(true);
    try {
      const ids = Array.from(selectedIds).map((id) => parseInt(id, 10));
      const result = await restoreFiles(ids);
      
      if (result.success) {
        setFiles((prev) => prev.filter((f) => !selectedIds.has(f.id)));
        setSelectedIds(new Set());
        setIsSelectMode(false);
      }
      console.log(result.message);
    } catch (err) {
      console.error("Failed to restore files:", err);
      setError("恢复文件失败");
    } finally {
      setIsDeleting(false);
    }
  };

  // 转换文件为 MediaItem 格式
  const convertToMediaItems = useCallback((files: any[], responseTotal?: number) => {
    const token = localStorage.getItem("token");
    const items: MediaItem[] = files.map((file) => ({
      id: String(file.id),
      thumbnail: file.thumbnail 
        ? `${file.thumbnail}?token=${token}`
        : file.url 
          ? `${file.url}?token=${token}` 
          : `https://picsum.photos/seed/${file.id}/400/400`,
      videoUrl: file.mime_type?.startsWith("video/") && file.url ? `${file.url}?token=${token}` : undefined,
      type: file.mime_type?.startsWith("video/") ? "video" : "image",
      title: file.file_name,
      date: new Date(file.created_at).toISOString().split("T")[0],
    }));
    
    const total = responseTotal ?? files.length;
    const totalPages = Math.ceil(total / PAGE_SIZE);
    
    return { items, total, totalPages };
  }, []);

  // 获取文件列表
  const fetchFiles = useCallback(async (reset: boolean = false, page: number = 1) => {
    try {
      setLoading(true);
      
      // 如果是回收站视图，使用回收站 API
      if (viewType === "trash") {
        const response = await getTrashList({ page, pageSize: PAGE_SIZE });
        const { items, total, totalPages: respTotalPages } = convertToMediaItems(response.files, response.total);
        
        if (reset) {
          setFiles(items);
          setCurrentPage(1);
        } else {
          setFiles((prev) => [...prev, ...items]);
        }
        setTotalPages(respTotalPages);
        setHasMore(page < respTotalPages);
        setError(null);
        return;
      }
      
      // 如果有搜索关键词，使用搜索参数
      if (searchQuery) {
        const searchUrl = `${FILES_API}?search=${encodeURIComponent(searchQuery)}&page_size=100`;
        const searchResponse = await authFetch(searchUrl);
        if (!searchResponse.ok) {
          throw new Error("Failed to search files");
        }
        const searchResult = await searchResponse.json();
        
        const files = (searchResult.files || []).slice(0, 100);
        const { items } = convertToMediaItems(files);
        setFiles(items);
        setTotalPages(1);
        setCurrentPage(1);
        setHasMore(false);
        return;
      }
      
      // 确定文件类型参数
      let fileTypeParam: string | undefined;
      switch (viewType) {
        case "videos":
          fileTypeParam = "video";
          break;
        case "photos":
        case "gifs":
        case "live-photos":
          fileTypeParam = "image";
          break;
        default:
          fileTypeParam = undefined;
      }
      
      const response = await getFileList({ 
        type: fileTypeParam, 
        page: page,
        pageSize: PAGE_SIZE 
      });
      
      const { items, total, totalPages: respTotalPages } = convertToMediaItems(response.files, response.total);
      
      if (reset) {
        setFiles(items);
        setCurrentPage(1);
      } else {
        setFiles((prev) => [...prev, ...items]);
      }
      setTotalPages(respTotalPages);
      setHasMore(page < respTotalPages);
      setError(null);
    } catch (err) {
      console.error("Failed to fetch files:", err);
      setError("Failed to load files");
    } finally {
      setLoading(false);
    }
  }, [viewType, searchQuery, convertToMediaItems]);

  // 加载更多数据
  const loadMoreData = useCallback(() => {
    if (!loading && currentPage < totalPages) {
      const nextPage = currentPage + 1;
      fetchFiles(false, nextPage);
      setCurrentPage(nextPage);
    }
  }, [loading, currentPage, totalPages, fetchFiles]);

  // 初始加载
  useEffect(() => {
    setFiles([]);
    setCurrentPage(1);
    setHasMore(true);
    setIsSelectMode(false);
    setSelectedIds(new Set());
    fetchFiles(true, 1);
  }, [viewType, searchQuery]);

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

  // 渲染媒体卡片
  const renderMediaCard = (item: MediaItem, showDate?: boolean) => {
    const isSelected = selectedIds.has(item.id);
    
    return (
      <Card
        key={item.id}
        shadow="sm"
        className={`aspect-square overflow-hidden group ${isSelected ? "ring-2 ring-primary" : ""}`}
      >
        <CardBody className="p-0">
          <div className="relative w-full h-full overflow-hidden">
            {/* 选择模式下的复选框覆盖层 */}
            {isSelectMode && (
              <div 
                className="absolute inset-0 z-20 cursor-pointer"
                onClick={() => toggleFileSelection(item.id)}
              >
                <div className="absolute top-1 left-1 md:top-2 md:left-2">
                  <div 
                    className={`w-5 h-5 md:w-6 md:h-6 rounded-full flex items-center justify-center ${
                      isSelected 
                        ? "bg-primary text-white" 
                        : "bg-white/80 dark:bg-black/50"
                    }`}
                  >
                    {isSelected && <Check className="w-4 h-4 md:w-5 md:h-5" />}
                  </div>
                </div>
                {/* 半透明遮罩 */}
                <div className={`absolute inset-0 ${isSelected ? "bg-primary/10" : "bg-transparent"}`} />
              </div>
            )}
            
            {item.type === "video" ? (
              <div 
                className="w-full h-full"
                onClick={() => !isSelectMode && item.videoUrl && openVideoPlayer(item.videoUrl, item.title)}
              >
                <VideoThumbnail src={item.videoUrl} poster={item.thumbnail} />
              </div>
            ) : (
              isSelectMode ? (
                <img
                  src={item.thumbnail}
                  alt={item.title}
                  className="w-full h-full object-cover"
                  loading="lazy"
                />
              ) : (
                <PhotoView src={item.thumbnail}>
                  <img
                    src={item.thumbnail}
                    alt={item.title}
                    className="w-full h-full object-cover transition-transform group-hover:scale-105 cursor-pointer"
                    loading="lazy"
                  />
                </PhotoView>
              )
            )}
            {item.type === "video" && (
              <div className="absolute top-1 right-1 md:top-2 md:right-2 z-10">
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
            {!isSelectMode && (
              <div className="absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors pointer-events-none" />
            )}
          </div>
        </CardBody>
      </Card>
    );
  };

  // 加载状态
  if (loading && files.length === 0) {
    return (
      <main
        id="scrollableDiv"
        ref={scrollRef}
        className="overflow-y-auto bg-default-50 dark:bg-default-900 transition-all duration-300 fixed left-0 right-0 md:left-64 top-16 bottom-0 md:bottom-0 p-4 md:p-6 pb-24 md:pb-6"
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
        id="scrollableDiv"
        ref={scrollRef}
        className="overflow-y-auto bg-default-50 dark:bg-default-900 transition-all duration-300 fixed left-0 right-0 md:left-64 top-16 bottom-0 md:bottom-0 p-4 md:p-6 pb-24 md:pb-6"
      >
        <div className="flex flex-col items-center justify-center h-full">
          <p className="text-danger">{error}</p>
        </div>
      </main>
    );
  }

  return (
    <main
      id="scrollableDiv"
      ref={scrollRef}
      className="overflow-y-auto bg-default-50 dark:bg-default-900 transition-all duration-300 fixed left-0 right-0 md:left-64 top-16 bottom-0 md:bottom-0 p-4 md:p-6 pb-24 md:pb-6"
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div>
          <h1 className="text-xl font-bold text-foreground">
            {getTitle()}{searchQuery && ` - 搜索: "${searchQuery}"`}
            {isSelectMode && selectedIds.size > 0 && ` (已选择 ${selectedIds.size} 项)`}
          </h1>
          <p className="text-sm text-default-500 mt-1">{files.length} 个项目</p>
        </div>
        
        <div className="flex items-center gap-2">
          {/* 选择模式下的操作按钮 */}
          {isSelectMode ? (
            <>
              <Button
                variant="flat"
                size="sm"
                onPress={toggleSelectAll}
              >
                {selectedIds.size === files.length ? "取消全选" : "全选"}
              </Button>
              {/* 回收站视图显示恢复和永久删除按钮 */}
              {viewType === "trash" ? (
                <>
                  <Button
                    color="success"
                    size="sm"
                    isDisabled={selectedIds.size === 0}
                    onPress={handleRestoreSelected}
                  >
                    恢复 ({selectedIds.size})
                  </Button>
                  <Button
                    color="danger"
                    size="sm"
                    startContent={<Trash2 className="w-4 h-4" />}
                    isDisabled={selectedIds.size === 0}
                    onPress={onDeleteModalOpen}
                  >
                    永久删除 ({selectedIds.size})
                  </Button>
                </>
              ) : (
                <Button
                  color="danger"
                  size="sm"
                  startContent={<Trash2 className="w-4 h-4" />}
                  isDisabled={selectedIds.size === 0}
                  onPress={onDeleteModalOpen}
                >
                  删除 ({selectedIds.size})
                </Button>
              )}
              <Button
                variant="light"
                size="sm"
                isIconOnly
                onPress={toggleSelectMode}
              >
                <X className="w-4 h-4" />
              </Button>
            </>
          ) : (
            <>
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
                  startContent={<Check className="w-4 h-4" />}
                  onPress={toggleSelectMode}
                >
                  选择
                </Button>
              </div>
            </>
          )}
        </div>
      </div>

      {/* Sort - Desktop only (not in select mode) */}
      {!isSelectMode && (
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
      )}

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
      ) : (
        <InfiniteScroll
          dataLength={files.length}
          next={loadMoreData}
          hasMore={hasMore}
          loader={
            <div className="flex justify-center items-center py-4">
              <Spinner size="sm" color="primary" />
              <span className="ml-2 text-sm text-default-500">加载更多...</span>
            </div>
          }
          endMessage={
            files.length >= PAGE_SIZE ? (
              <div className="flex justify-center items-center py-4 mt-4 text-sm text-default-400">
                已加载全部 {files.length} 个项目
              </div>
            ) : null
          }
          scrollableTarget="scrollableDiv"
          style={{ overflow: 'visible' }}
        >
          {viewMode === "grid" ? (
            /* Grid View */
            <PhotoProvider>
              <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-2 md:gap-4">
                {files.map((item) => renderMediaCard(item))}
              </div>
            </PhotoProvider>
          ) : (
            /* List View */
            <PhotoProvider>
              <div className="space-y-6">
                {groupMediaByDate(files).map(([date, items]) => (
                  <div key={date}>
                    {/* Date Header */}
                    <div className="sticky top-0 z-10 bg-default-50/95 dark:bg-default-900/95 backdrop-blur-sm py-2 mb-3">
                      <h3 className="text-sm font-semibold text-default-600">{date}</h3>
                    </div>
                    
                    {/* Media Grid for this date */}
                    <div className="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 gap-2">
                      {items.map((item) => renderMediaCard(item))}
                    </div>
                  </div>
                ))}
              </div>
            </PhotoProvider>
          )}
        </InfiniteScroll>
      )}

      {/* 视频播放器模态框 */}
      <VideoPlayerModal
        isOpen={isVideoPlayerOpen}
        onClose={closeVideoPlayer}
        videoUrl={currentVideoUrl}
        title={currentVideoTitle}
      />

      {/* 删除确认对话框 */}
      <Modal isOpen={isDeleteModalOpen} onClose={onDeleteModalClose}>
        <ModalContent>
          <ModalHeader>确认删除</ModalHeader>
          <ModalBody>
            <p>确定要删除选中的 {selectedIds.size} 个文件吗？</p>
            <p className="text-sm text-default-500">文件将被移动到回收站，您可以从回收站恢复。</p>
          </ModalBody>
          <ModalFooter>
            <Button variant="light" onPress={onDeleteModalClose}>
              取消
            </Button>
            <Button 
              color="danger" 
              onPress={handleDeleteSelected}
              isLoading={isDeleting}
            >
              删除
            </Button>
          </ModalFooter>
        </ModalContent>
      </Modal>
    </main>
  );
}
