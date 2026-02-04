import React, { useState, useCallback, useRef } from "react";
import {
  Button,
  Modal,
  ModalContent,
  ModalHeader,
  ModalBody,
  ModalFooter,
  Progress,
  Card,
  CardBody,
  Chip,
  Image,
} from "@heroui/react";
import { Upload, X, Check, AlertCircle, FileImage, FileVideo } from "lucide-react";
import { createXXHash3, type IHasher } from "hash-wasm";

// API to check if hash exists
async function checkHashExists(hash: string): Promise<boolean> {
  try {
    const response = await fetch(`/api/files/check-hash?hash=${encodeURIComponent(hash)}`);
    if (!response.ok) return false;
    const data = await response.json();
    return data.exists;
  } catch {
    return false;
  }
}

// Calculate file hash using xxHash3 with chunked reading for large files
async function calculateFileHash(
  file: File,
  onProgress?: (progress: number) => void
): Promise<string> {
  const xxhash3: IHasher = await createXXHash3();
  
  const chunkSize = 10 * 1024 * 1024; // 10MB chunks
  let offset = 0;
  
  while (offset < file.size) {
    const chunk = file.slice(offset, offset + chunkSize);
    const chunkArray = new Uint8Array(await chunk.arrayBuffer());
    xxhash3.update(chunkArray);
    
    offset += chunkSize;
    
    // Report progress
    if (onProgress) {
      onProgress(Math.min((offset / file.size) * 100, 99));
    }
  }
  
  const hash = xxhash3.digest();
  return hash;
}

interface UploadFile {
  file: File;
  preview?: string;
  hash: string;
  status: "pending" | "hashing" | "checking" | "uploading" | "success" | "skipped" | "error";
  progress: number;
  error?: string;
}

interface UploadModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function UploadModal({ isOpen, onClose }: UploadModalProps) {
  const [files, setFiles] = useState<UploadFile[]>([]);
  const [isUploading, setIsUploading] = useState(false);
  const processingRef = useRef(false);

  // Handle file selection
  const handleFileSelect = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    if (processingRef.current) return;
    
    const selectedFiles = Array.from(e.target.files || []);
    
    const newFiles: UploadFile[] = selectedFiles.map((file) => ({
      file,
      preview: file.type.startsWith("image/") ? URL.createObjectURL(file) : undefined,
      hash: "",
      status: "pending",
      progress: 0,
    }));
    
    setFiles((prev) => [...prev, ...newFiles]);
    
    // Start processing files
    processingRef.current = true;
    await processFiles(newFiles);
    processingRef.current = false;
  }, []);

  // Process files - hash, check, then upload
  const processFiles = async (filesToProcess: UploadFile[]) => {
    for (const uploadFile of filesToProcess) {
      try {
        // Update status to hashing
        setFiles((prev) =>
          prev.map((f) =>
            f.file === uploadFile.file ? { ...f, status: "hashing" } : f
          )
        );

        // Calculate hash with progress tracking
        const hash = await calculateFileHash(uploadFile.file, (progress) => {
          setFiles((prev) =>
            prev.map((f) =>
              f.file === uploadFile.file ? { ...f, progress } : f
            )
          );
        });
        
        setFiles((prev) =>
          prev.map((f) =>
            f.file === uploadFile.file ? { ...f, hash: hash.substring(0, 16) + "...", progress: 100 } : f
          )
        );

        // Update status to checking
        setFiles((prev) =>
          prev.map((f) =>
            f.file === uploadFile.file ? { ...f, status: "checking" } : f
          )
        );

        // Check if hash exists
        const exists = await checkHashExists(hash);
        
        if (exists) {
          // Hash exists, skip upload
          setFiles((prev) =>
            prev.map((f) =>
              f.file === uploadFile.file ? { ...f, status: "skipped" } : f
            )
          );
          continue;
        }

        // Hash doesn't exist, upload file
        setFiles((prev) =>
          prev.map((f) =>
            f.file === uploadFile.file ? { ...f, status: "uploading", progress: 0 } : f
          )
        );

        // Upload file
        const formData = new FormData();
        formData.append("file", uploadFile.file);
        formData.append("hash", hash);

        const response = await fetch("/api/files/upload", {
          method: "POST",
          body: formData,
        });

        if (!response.ok) throw new Error("Upload failed");

        setFiles((prev) =>
          prev.map((f) =>
            f.file === uploadFile.file ? { ...f, status: "success", progress: 100 } : f
          )
        );
      } catch (error) {
        setFiles((prev) =>
          prev.map((f) =>
            f.file === uploadFile.file
              ? { ...f, status: "error", error: "Upload failed" }
              : f
          )
        );
      }
    }
    
    setIsUploading(false);
  };

  // Remove file from list
  const removeFile = (index: number) => {
    setFiles((prev) => {
      const newFiles = [...prev];
      if (newFiles[index].preview) {
        URL.revokeObjectURL(newFiles[index].preview!);
      }
      newFiles.splice(index, 1);
      return newFiles;
    });
  };

  // Get status icon
  const getStatusIcon = (status: UploadFile["status"]) => {
    switch (status) {
      case "pending":
        return <Chip size="sm" variant="flat">等待</Chip>;
      case "hashing":
        return <Chip size="sm" color="secondary" variant="flat">计算哈希...</Chip>;
      case "checking":
        return <Chip size="sm" color="warning" variant="flat">检查中...</Chip>;
      case "uploading":
        return <Chip size="sm" color="primary" variant="flat">上传中...</Chip>;
      case "success":
        return <Chip size="sm" color="success" variant="flat" startContent={<Check className="w-3 h-3" />}>成功</Chip>;
      case "skipped":
        return <Chip size="sm" color="default" variant="flat" startContent={<Check className="w-3 h-3" />}>已存在</Chip>;
      case "error":
        return <Chip size="sm" color="danger" variant="flat" startContent={<AlertCircle className="w-3 h-3" />}>失败</Chip>;
    }
  };

  // Reset and close
  const handleClose = () => {
    files.forEach((f) => {
      if (f.preview) URL.revokeObjectURL(f.preview);
    });
    setFiles([]);
    onClose();
  };

  const completedCount = files.filter((f) => f.status === "success" || f.status === "skipped").length;
  const totalCount = files.length;

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleClose}
      size="2xl"
      scrollBehavior="inside"
      classNames={{
        base: "max-h-[80vh]",
      }}
    >
      <ModalContent>
        <ModalHeader className="flex flex-col gap-2">
          <div className="flex items-center gap-2">
            <Upload className="w-5 h-5" />
            <span>上传文件</span>
          </div>
          {totalCount > 0 && (
            <Progress
              value={(completedCount / totalCount) * 100}
              size="sm"
              color="primary"
              className="mt-2"
            />
          )}
        </ModalHeader>
        
        <ModalBody>
          {/* Upload area */}
          <div className="border-2 border-dashed border-divider rounded-xl p-8 text-center hover:border-primary transition-colors cursor-pointer relative">
            <input
              type="file"
              multiple
              accept="image/*,video/*"
              onChange={handleFileSelect}
              className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
            />
            <Upload className="w-10 h-10 mx-auto text-default-400 mb-3" />
            <p className="text-lg font-medium mb-1">点击或拖拽文件到此处</p>
            <p className="text-sm text-default-500">支持图片和视频文件</p>
          </div>

          {/* File list */}
          {files.length > 0 && (
            <div className="space-y-3 mt-4">
              {files.map((uploadFile, index) => (
                <Card key={index} shadow="sm">
                  <CardBody className="flex-row items-center gap-4 p-3">
                    {/* Preview */}
                    {uploadFile.preview ? (
                      <div className="w-12 h-12 rounded-lg overflow-hidden flex-shrink-0">
                        <Image
                          src={uploadFile.preview}
                          alt={uploadFile.file.name}
                          className="w-full h-full object-cover"
                        />
                      </div>
                    ) : (
                      <div className="w-12 h-12 rounded-lg bg-default-100 flex items-center justify-center flex-shrink-0">
                        {uploadFile.file.type.startsWith("video/") ? (
                          <FileVideo className="w-6 h-6 text-default-400" />
                        ) : (
                          <FileImage className="w-6 h-6 text-default-400" />
                        )}
                      </div>
                    )}
                    
                    {/* File info */}
                    <div className="flex-1 min-w-0">
                      <p className="font-medium truncate">{uploadFile.file.name}</p>
                      <p className="text-sm text-default-500">
                        {(uploadFile.file.size / 1024 / 1024).toFixed(2)} MB
                      </p>
                      {uploadFile.hash && (
                        <p className="text-xs text-default-400 font-mono mt-1">
                          xxHash3: {uploadFile.hash}
                        </p>
                      )}
                    </div>
                    
                    {/* Status */}
                    <div className="flex-shrink-0">
                      {(uploadFile.status === "uploading" || uploadFile.status === "hashing") ? (
                        <Progress
                          size="sm"
                          value={uploadFile.progress}
                          color="primary"
                          className="w-20"
                        />
                      ) : (
                        getStatusIcon(uploadFile.status)
                      )}
                    </div>
                    
                    {/* Remove button */}
                    <Button
                      isIconOnly
                      variant="light"
                      size="sm"
                      onPress={() => removeFile(index)}
                    >
                      <X className="w-4 h-4" />
                    </Button>
                  </CardBody>
                </Card>
              ))}
            </div>
          )}
        </ModalBody>
        
        <ModalFooter>
          <Button variant="light" onPress={handleClose}>
            关闭
          </Button>
        </ModalFooter>
      </ModalContent>
    </Modal>
  );
}
