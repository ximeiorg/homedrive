import React, { useState, useEffect, useCallback, useMemo } from "react";
import { useNavigate } from "react-router";
import {
  Card,
  CardBody,
  Button,
  Modal,
  ModalContent,
  ModalHeader,
  ModalBody,
  ModalFooter,
  Input,
  Textarea,
  useDisclosure,
  Spinner,
  Chip,
} from "@heroui/react";
import {
  Plus,
  FolderOpen,
  Trash2,
  Edit3,
  ArrowLeft,
  Image as ImageIcon,
  CheckCircle,
} from "lucide-react";
import { PhotoProvider, PhotoView } from "react-photo-view";
import "react-photo-view/dist/react-photo-view.css";

import {
  getAlbumList,
  createAlbum,
  deleteAlbum,
  updateAlbum,
  getAlbumFiles,
  addFilesToAlbum,
  removeFilesFromAlbum,
  getFileList,
  type AlbumListItem,
  type AlbumFileInfo,
} from "../api";
import { useAuth } from "../auth-context";
import { TopBar } from "../components/TopBar";
import { Sidebar } from "../components/Sidebar";
import { UploadModal } from "../components/UploadModal";

export function meta() {
  return [
    { title: "相册 - HomeDrive" },
    { name: "description", content: "Your photo albums" },
  ];
}

// 辅助函数：为URL添加token参数
function addTokenToUrl(url: string | null | undefined, token: string | null): string {
  if (!url) return "";
  if (!token) return url;
  const separator = url.includes("?") ? "&" : "?";
  return `${url}${separator}token=${token}`;
}

// 相册卡片组件
function AlbumCard({
  album,
  onClick,
  onLongPress,
  token,
}: {
  album: AlbumListItem;
  onClick: () => void;
  onLongPress: () => void;
  token: string | null;
}) {
  const coverUrl = addTokenToUrl(album.cover_url, token);
  
  return (
    <Card
      isPressable
      className="w-full aspect-square overflow-hidden group"
      onPress={onClick}
      onContextMenu={(e) => {
        e.preventDefault();
        onLongPress();
      }}
    >
      <CardBody className="p-0 relative">
        {coverUrl ? (
          <img
            src={coverUrl}
            alt={album.name}
            className="w-full h-full object-cover transition-transform group-hover:scale-105"
          />
        ) : (
          <div className="w-full h-full bg-default-100 flex items-center justify-center">
            <FolderOpen className="w-16 h-16 text-default-300" />
          </div>
        )}
        <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/70 to-transparent p-3">
          <h3 className="text-white font-semibold truncate">{album.name}</h3>
          <p className="text-white/70 text-sm">{album.file_count} 张照片</p>
        </div>
      </CardBody>
    </Card>
  );
}

// 文件选择器组件
function FileSelector({
  isOpen,
  onClose,
  onConfirm,
  excludeIds,
  token,
}: {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: (fileIds: number[]) => void;
  excludeIds: number[];
  token: string | null;
}) {
  const [files, setFiles] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());

  useEffect(() => {
    if (isOpen) {
      setLoading(true);
      setSelectedIds(new Set());
      getFileList({ type: "image", pageSize: 100 })
        .then((result) => {
          // 过滤掉已经在相册中的文件
          const filteredFiles = result.files.filter(
            (f) => !excludeIds.includes(f.id)
          );
          setFiles(filteredFiles);
        })
        .catch(console.error)
        .finally(() => setLoading(false));
    }
  }, [isOpen, excludeIds]);

  const toggleSelection = (id: number) => {
    const newSet = new Set(selectedIds);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    setSelectedIds(newSet);
  };

  const handleConfirm = () => {
    onConfirm(Array.from(selectedIds));
    onClose();
  };

  if (!isOpen) return null;

  return (
    <Modal isOpen={isOpen} onClose={onClose} size="4xl" scrollBehavior="inside">
      <ModalContent>
        <ModalHeader>选择要添加的照片</ModalHeader>
        <ModalBody>
          {loading ? (
            <div className="flex justify-center py-8">
              <Spinner size="lg" />
            </div>
          ) : files.length === 0 ? (
            <div className="text-center py-8 text-default-500">
              没有可添加的照片
            </div>
          ) : (
            <div className="grid grid-cols-4 md:grid-cols-6 gap-2">
              {files.map((file) => {
                const thumbnailUrl = addTokenToUrl(file.thumbnail, token) || 
                  addTokenToUrl(file.url, token) ||
                  `https://picsum.photos/seed/${file.id}/200/200`;
                
                return (
                  <div
                    key={file.id}
                    className={`relative aspect-square cursor-pointer rounded-lg overflow-hidden border-2 transition-all ${
                      selectedIds.has(file.id)
                        ? "border-primary"
                        : "border-transparent"
                    }`}
                    onClick={() => toggleSelection(file.id)}
                  >
                    <img
                      src={thumbnailUrl}
                      alt={file.file_name}
                      className="w-full h-full object-cover"
                    />
                    {selectedIds.has(file.id) && (
                      <div className="absolute inset-0 bg-primary/20 flex items-center justify-center">
                        <CheckCircle className="w-8 h-8 text-primary" />
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </ModalBody>
        <ModalFooter>
          <Button variant="flat" onPress={onClose}>
            取消
          </Button>
          <Button
            color="primary"
            onPress={handleConfirm}
            isDisabled={selectedIds.size === 0}
          >
            添加 {selectedIds.size > 0 && `(${selectedIds.size})`}
          </Button>
        </ModalFooter>
      </ModalContent>
    </Modal>
  );
}

export default function Albums() {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isUploadOpen, setIsUploadOpen] = useState(false);
  const { isAuthenticated, member, token } = useAuth();
  const navigate = useNavigate();

  // 相册列表状态
  const [albums, setAlbums] = useState<AlbumListItem[]>([]);
  const [albumsLoading, setAlbumsLoading] = useState(true);
  const [albumsError, setAlbumsError] = useState<string | null>(null);

  // 当前查看的相册
  const [currentAlbum, setCurrentAlbum] = useState<AlbumListItem | null>(null);
  const [albumFiles, setAlbumFiles] = useState<AlbumFileInfo[]>([]);
  const [albumFilesLoading, setAlbumFilesLoading] = useState(false);

  // 创建相册对话框
  const createAlbumDisclosure = useDisclosure();
  const [newAlbumName, setNewAlbumName] = useState("");
  const [newAlbumDescription, setNewAlbumDescription] = useState("");
  const [isCreating, setIsCreating] = useState(false);

  // 删除相册对话框
  const deleteAlbumDisclosure = useDisclosure();
  const [albumToDelete, setAlbumToDelete] = useState<AlbumListItem | null>(
    null
  );
  const [isDeleting, setIsDeleting] = useState(false);

  // 编辑相册对话框
  const editAlbumDisclosure = useDisclosure();
  const [albumToEdit, setAlbumToEdit] = useState<AlbumListItem | null>(null);
  const [editAlbumName, setEditAlbumName] = useState("");
  const [editAlbumDescription, setEditAlbumDescription] = useState("");
  const [isEditing, setIsEditing] = useState(false);

  // 文件选择器
  const [isFileSelectorOpen, setIsFileSelectorOpen] = useState(false);

  // 客户端登录检查
  useEffect(() => {
    if (!isAuthenticated) {
      navigate("/login", { replace: true });
    }
  }, [isAuthenticated, navigate]);

  // 加载相册列表
  const loadAlbums = useCallback(async () => {
    if (!member?.id) return;

    try {
      setAlbumsLoading(true);
      setAlbumsError(null);
      const result = await getAlbumList(member.id);
      setAlbums(result.albums);
    } catch (err) {
      console.error("Failed to load albums:", err);
      setAlbumsError("加载相册失败");
    } finally {
      setAlbumsLoading(false);
    }
  }, [member?.id]);

  // 初始加载相册
  useEffect(() => {
    if (member?.id) {
      loadAlbums();
    }
  }, [member?.id, loadAlbums]);

  // 加载相册文件
  const loadAlbumFiles = useCallback(
    async (albumId: number) => {
      if (!member?.id) return;

      try {
        setAlbumFilesLoading(true);
        const result = await getAlbumFiles(member.id, albumId);
        setAlbumFiles(result.files);
      } catch (err) {
        console.error("Failed to load album files:", err);
      } finally {
        setAlbumFilesLoading(false);
      }
    },
    [member?.id]
  );

  // 打开相册
  const openAlbum = (album: AlbumListItem) => {
    setCurrentAlbum(album);
    loadAlbumFiles(album.id);
  };

  // 关闭相册
  const closeAlbum = () => {
    setCurrentAlbum(null);
    setAlbumFiles([]);
  };

  // 创建相册
  const handleCreateAlbum = async () => {
    if (!member?.id || !newAlbumName.trim()) return;

    try {
      setIsCreating(true);
      await createAlbum(member.id, {
        name: newAlbumName.trim(),
        description: newAlbumDescription.trim() || undefined,
      });
      createAlbumDisclosure.onClose();
      setNewAlbumName("");
      setNewAlbumDescription("");
      loadAlbums();
    } catch (err) {
      console.error("Failed to create album:", err);
    } finally {
      setIsCreating(false);
    }
  };

  // 删除相册
  const handleDeleteAlbum = async () => {
    if (!member?.id || !albumToDelete) return;

    try {
      setIsDeleting(true);
      await deleteAlbum(member.id, albumToDelete.id);
      deleteAlbumDisclosure.onClose();
      setAlbumToDelete(null);
      loadAlbums();
    } catch (err) {
      console.error("Failed to delete album:", err);
    } finally {
      setIsDeleting(false);
    }
  };

  // 编辑相册
  const handleEditAlbum = async () => {
    if (!member?.id || !albumToEdit || !editAlbumName.trim()) return;

    try {
      setIsEditing(true);
      await updateAlbum(member.id, albumToEdit.id, {
        name: editAlbumName.trim(),
        description: editAlbumDescription.trim() || undefined,
      });
      editAlbumDisclosure.onClose();
      setAlbumToEdit(null);
      loadAlbums();
    } catch (err) {
      console.error("Failed to edit album:", err);
    } finally {
      setIsEditing(false);
    }
  };

  // 显示删除确认对话框
  const showDeleteDialog = (album: AlbumListItem) => {
    setAlbumToDelete(album);
    deleteAlbumDisclosure.onOpen();
  };

  // 显示编辑对话框
  const showEditDialog = (album: AlbumListItem) => {
    setAlbumToEdit(album);
    setEditAlbumName(album.name);
    setEditAlbumDescription(album.description || "");
    editAlbumDisclosure.onOpen();
  };

  // 添加文件到相册
  const handleAddFiles = async (fileIds: number[]) => {
    if (!member?.id || !currentAlbum || fileIds.length === 0) return;

    try {
      await addFilesToAlbum(member.id, currentAlbum.id, fileIds);
      loadAlbumFiles(currentAlbum.id);
      loadAlbums(); // 刷新相册列表以更新文件计数
    } catch (err) {
      console.error("Failed to add files to album:", err);
    }
  };

  // 从相册移除文件
  const handleRemoveFile = async (fileId: number) => {
    if (!member?.id || !currentAlbum) return;

    try {
      await removeFilesFromAlbum(member.id, currentAlbum.id, [fileId]);
      loadAlbumFiles(currentAlbum.id);
      loadAlbums();
    } catch (err) {
      console.error("Failed to remove file from album:", err);
    }
  };

  // 为相册文件添加token
  const albumFilesWithToken = useMemo(() => {
    return albumFiles.map((file) => ({
      ...file,
      thumbnail: addTokenToUrl(file.thumbnail, token),
      url: addTokenToUrl(file.url, token),
    }));
  }, [albumFiles, token]);

  // 未登录时不显示内容
  if (!isAuthenticated || !member) {
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
        selectedKey="albums"
        isMenuOpen={isMenuOpen}
        onMenuClose={() => setIsMenuOpen(false)}
      />

      {/* 主内容区域 */}
      <main className="overflow-y-auto bg-default-50 dark:bg-default-900 transition-all duration-300 fixed left-0 right-0 md:left-64 top-16 bottom-0 p-4 md:p-6 pb-24 md:pb-6">
        {/* 相册详情视图 */}
        {currentAlbum ? (
          <div>
            {/* 顶部导航 */}
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-3">
                <Button
                  isIconOnly
                  variant="flat"
                  onPress={closeAlbum}
                >
                  <ArrowLeft className="w-5 h-5" />
                </Button>
                <div>
                  <h1 className="text-xl font-bold text-foreground">
                    {currentAlbum.name}
                  </h1>
                  {currentAlbum.description && (
                    <p className="text-sm text-default-500">
                      {currentAlbum.description}
                    </p>
                  )}
                </div>
              </div>
              <div className="flex items-center gap-2">
                <Button
                  color="primary"
                  size="sm"
                  startContent={<Plus className="w-4 h-4" />}
                  onPress={() => setIsFileSelectorOpen(true)}
                >
                  添加照片
                </Button>
                <Button
                  variant="flat"
                  size="sm"
                  startContent={<Edit3 className="w-4 h-4" />}
                  onPress={() => showEditDialog(currentAlbum)}
                >
                  编辑
                </Button>
              </div>
            </div>

            {/* 照片网格 */}
            {albumFilesLoading ? (
              <div className="flex justify-center py-8">
                <Spinner size="lg" />
              </div>
            ) : albumFiles.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-16">
                <ImageIcon className="w-16 h-16 text-default-300 mb-4" />
                <p className="text-default-500 mb-4">相册中还没有照片</p>
                <Button
                  color="primary"
                  startContent={<Plus className="w-4 h-4" />}
                  onPress={() => setIsFileSelectorOpen(true)}
                >
                  添加照片
                </Button>
              </div>
            ) : (
              <PhotoProvider>
                <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-2">
                  {albumFilesWithToken.map((file) => (
                    <div
                      key={file.id}
                      className="relative aspect-square group"
                    >
                      <PhotoView src={file.url || ""}>
                        <img
                          src={
                            file.thumbnail ||
                            file.url ||
                            `https://picsum.photos/seed/${file.id}/200/200`
                          }
                          alt={file.file_name}
                          className="w-full h-full object-cover rounded-lg cursor-pointer transition-transform group-hover:scale-105"
                        />
                      </PhotoView>
                      {/* 删除按钮 */}
                      <Button
                        isIconOnly
                        size="sm"
                        color="danger"
                        variant="flat"
                        className="absolute top-1 right-1 opacity-0 group-hover:opacity-100 transition-opacity"
                        onPress={() => handleRemoveFile(file.id)}
                      >
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                  ))}
                </div>
              </PhotoProvider>
            )}
          </div>
        ) : (
          // 相册列表视图
          <div>
            {/* 顶部标题 */}
            <div className="flex items-center justify-between mb-4">
              <div>
                <h1 className="text-xl font-bold text-foreground">相册</h1>
                <p className="text-sm text-default-500 mt-1">
                  {albums.length} 个相册
                </p>
              </div>
              <Button
                color="primary"
                size="sm"
                startContent={<Plus className="w-4 h-4" />}
                onPress={createAlbumDisclosure.onOpen}
              >
                创建相册
              </Button>
            </div>

            {/* 加载状态 */}
            {albumsLoading ? (
              <div className="flex justify-center py-8">
                <Spinner size="lg" />
              </div>
            ) : albumsError ? (
              <div className="flex flex-col items-center justify-center py-16">
                <p className="text-danger mb-4">{albumsError}</p>
                <Button color="primary" onPress={loadAlbums}>
                  重试
                </Button>
              </div>
            ) : albums.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-16">
                <FolderOpen className="w-16 h-16 text-default-300 mb-4" />
                <p className="text-default-500 mb-4">还没有相册</p>
                <Button
                  color="primary"
                  startContent={<Plus className="w-4 h-4" />}
                  onPress={createAlbumDisclosure.onOpen}
                >
                  创建第一个相册
                </Button>
              </div>
            ) : (
              <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
                {albums.map((album) => (
                  <div key={album.id} className="relative group">
                    <AlbumCard
                      album={album}
                      onClick={() => openAlbum(album)}
                      onLongPress={() => showDeleteDialog(album)}
                      token={token}
                    />
                    {/* 操作按钮 */}
                    <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity flex gap-1">
                      <Button
                        isIconOnly
                        size="sm"
                        variant="flat"
                        color="default"
                        className="bg-background/80 backdrop-blur-sm"
                        onPress={(e) => {
                          e.stopPropagation();
                          showEditDialog(album);
                        }}
                      >
                        <Edit3 className="w-4 h-4" />
                      </Button>
                      <Button
                        isIconOnly
                        size="sm"
                        variant="flat"
                        color="danger"
                        className="bg-background/80 backdrop-blur-sm"
                        onPress={(e) => {
                          e.stopPropagation();
                          showDeleteDialog(album);
                        }}
                      >
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </main>

      {/* Upload Modal */}
      <UploadModal isOpen={isUploadOpen} onClose={() => setIsUploadOpen(false)} />

      {/* 创建相册对话框 */}
      <Modal
        isOpen={createAlbumDisclosure.isOpen}
        onClose={createAlbumDisclosure.onClose}
      >
        <ModalContent>
          <ModalHeader>创建相册</ModalHeader>
          <ModalBody>
            <Input
              label="相册名称"
              placeholder="输入相册名称"
              value={newAlbumName}
              onValueChange={setNewAlbumName}
              isRequired
            />
            <Textarea
              label="相册描述"
              placeholder="输入相册描述（可选）"
              value={newAlbumDescription}
              onValueChange={setNewAlbumDescription}
            />
          </ModalBody>
          <ModalFooter>
            <Button variant="flat" onPress={createAlbumDisclosure.onClose}>
              取消
            </Button>
            <Button
              color="primary"
              onPress={handleCreateAlbum}
              isLoading={isCreating}
              isDisabled={!newAlbumName.trim()}
            >
              创建
            </Button>
          </ModalFooter>
        </ModalContent>
      </Modal>

      {/* 删除相册确认对话框 */}
      <Modal
        isOpen={deleteAlbumDisclosure.isOpen}
        onClose={deleteAlbumDisclosure.onClose}
      >
        <ModalContent>
          <ModalHeader>删除相册</ModalHeader>
          <ModalBody>
            <p>
              确定要删除相册 "{albumToDelete?.name}" 吗？此操作不可恢复。
            </p>
          </ModalBody>
          <ModalFooter>
            <Button variant="flat" onPress={deleteAlbumDisclosure.onClose}>
              取消
            </Button>
            <Button
              color="danger"
              onPress={handleDeleteAlbum}
              isLoading={isDeleting}
            >
              删除
            </Button>
          </ModalFooter>
        </ModalContent>
      </Modal>

      {/* 编辑相册对话框 */}
      <Modal
        isOpen={editAlbumDisclosure.isOpen}
        onClose={editAlbumDisclosure.onClose}
      >
        <ModalContent>
          <ModalHeader>编辑相册</ModalHeader>
          <ModalBody>
            <Input
              label="相册名称"
              placeholder="输入相册名称"
              value={editAlbumName}
              onValueChange={setEditAlbumName}
              isRequired
            />
            <Textarea
              label="相册描述"
              placeholder="输入相册描述（可选）"
              value={editAlbumDescription}
              onValueChange={setEditAlbumDescription}
            />
          </ModalBody>
          <ModalFooter>
            <Button variant="flat" onPress={editAlbumDisclosure.onClose}>
              取消
            </Button>
            <Button
              color="primary"
              onPress={handleEditAlbum}
              isLoading={isEditing}
              isDisabled={!editAlbumName.trim()}
            >
              保存
            </Button>
          </ModalFooter>
        </ModalContent>
      </Modal>

      {/* 文件选择器 */}
      <FileSelector
        isOpen={isFileSelectorOpen}
        onClose={() => setIsFileSelectorOpen(false)}
        onConfirm={handleAddFiles}
        excludeIds={albumFiles.map((f) => f.id)}
        token={token}
      />
    </div>
  );
}
