const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || "http://127.0.0.1:2300";
const API_BASE = `${API_BASE_URL}/api`;
const MEMBERS_API = `${API_BASE_URL}/api/members`;
const FILES_API = `${API_BASE_URL}/api/files`;
const TASKS_API = `${API_BASE_URL}/api/tasks`;
const SYSTEM_API = `${API_BASE_URL}/api/system`;
const AUTH_API = `${API_BASE_URL}/api/auth`;

export { FILES_API, TASKS_API, SYSTEM_API, AUTH_API };

export interface IsEmptyResponse {
  is_empty: boolean;
}

export interface InitAdminRequest {
  username: string;
  password: string;
  storage_tag: string;
}

export interface InitAdminResponse {
  success: boolean;
  message: string;
  member?: {
    id: number;
    username: string;
    avatar: string | null;
    storage_tag: string;
    created_at: string;
  };
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  token: string;
  member: {
    id: number;
    username: string;
    avatar: string | null;
    storage_tag: string;
    created_at: string;
  };
}

// 带认证的 fetch 函数
export async function authFetch(
  url: string,
  options: RequestInit = {}
): Promise<Response> {
  const token = localStorage.getItem("token");
  
  const headers: HeadersInit = {
    ...(options.headers || {}),
  };
  
  if (token) {
    (headers as Record<string, string>)["Authorization"] = `Bearer ${token}`;
  }
  
  return fetch(url, {
    ...options,
    headers,
  });
}

// 检查 member 表是否为空（公开接口）
export async function checkMembersEmpty(): Promise<IsEmptyResponse> {
  const response = await fetch(`${API_BASE}/empty`);
  if (!response.ok) {
    throw new Error("Failed to check members empty status");
  }
  return response.json();
}

// 初始化管理员（公开接口）
export async function initAdmin(data: InitAdminRequest): Promise<InitAdminResponse> {
  const response = await fetch(`${API_BASE}/init`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    throw new Error("Failed to initialize admin");
  }
  return response.json();
}

// 登录（公开接口）
export async function login(data: LoginRequest): Promise<LoginResponse> {
  const response = await fetch(`${AUTH_API}/login`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    throw new Error("Login failed");
  }
  return response.json();
}

// 检查文件 hash 是否存在（需要认证）
export async function checkFileHashExists(hash: string): Promise<{ exists: boolean }> {
  const response = await authFetch(`${FILES_API}/check-hash?hash=${encodeURIComponent(hash)}`);
  if (!response.ok) {
    throw new Error("Failed to check file hash");
  }
  return response.json();
}

// 获取文件列表（需要认证）
export async function getFileList(params?: {
  type?: string;
  page?: number;
  pageSize?: number;
}): Promise<{
  files: Array<{
    id: number;
    file_name: string;
    description: string;
    file_size: number | null;
    mime_type: string | null;
    thumbnail: string | null;
    url: string | null;
    created_at: string;
    updated_at: string;
  }>;
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}> {
  const searchParams = new URLSearchParams();
  if (params?.type) {
    searchParams.set("file_type", params.type);
  }
  if (params?.page) {
    searchParams.set("page", params.page.toString());
  }
  if (params?.pageSize) {
    searchParams.set("page_size", params.pageSize.toString());
  }
  
  const url = `${FILES_API}${searchParams.toString() ? `?${searchParams.toString()}` : ""}`;
  const response = await authFetch(url);
  if (!response.ok) {
    throw new Error("Failed to get file list");
  }
  return response.json();
}

// 成员类型
export interface MemberResponse {
  id: number;
  username: string;
  avatar: string | null;
  storage_tag: string;
  storage_used: number;
  storage_total: number;
  last_active: string | null;
  status: "online" | "offline" | "away";
  created_at: string;
}

// 成员列表响应
export interface MemberListResponse {
  members: MemberResponse[];
  total: number;
  page: number;
  page_size: number;
}

// 获取成员列表（需要认证）
export async function getMemberList(): Promise<MemberListResponse> {
  const response = await authFetch(`${API_BASE}/members`);
  if (!response.ok) {
    throw new Error("Failed to get member list");
  }
  return response.json();
}

// 获取单个成员详情（需要认证）
export async function getMemberDetail(memberId: number): Promise<MemberResponse> {
  const response = await authFetch(`${API_BASE}/members/${memberId}`);
  if (!response.ok) {
    throw new Error("Failed to get member detail");
  }
  return response.json();
}

// 获取任务消息（需要认证）
export async function getTaskMessages(taskId: string): Promise<{
  messages: Array<{
    id: number;
    task_id: string;
    message: string;
    progress: number;
    created_at: string;
  }>;
}> {
  const response = await authFetch(`${FILES_API}/task-messages/${taskId}`);
  if (!response.ok) {
    throw new Error("Failed to get task messages");
  }
  return response.json();
}

// 任务类型
export type TaskType = "upload" | "download" | "process" | "sync";
export type TaskStatus = "pending" | "processing" | "completed" | "failed";

// 任务项接口
export interface TaskItem {
  id: number;
  task_type: string;
  status: TaskStatus;
  progress: number;
  message: string;
  created_at: string;
  updated_at: string;
  completed_at: string | null;
}

// 任务列表响应
export interface TaskListResponse {
  tasks: TaskItem[];
}

// 获取任务列表（需要认证）
export async function getTaskList(): Promise<TaskListResponse> {
  const response = await authFetch(`${TASKS_API}`);
  if (!response.ok) {
    throw new Error("Failed to get task list");
  }
  return response.json();
}

// 获取单个任务详情（需要认证）
export async function getTaskDetail(taskId: number): Promise<TaskItem> {
  const response = await authFetch(`${TASKS_API}/${taskId}`);
  if (!response.ok) {
    throw new Error("Failed to get task detail");
  }
  return response.json();
}

// 同步文件请求
export interface SyncFilesRequest {
  path?: string;
}

// 同步文件响应
export interface SyncFilesResponse {
  success: boolean;
  task_id: number;
  message: string;
}

// 同步文件（需要认证）
export async function syncFiles(data: SyncFilesRequest): Promise<SyncFilesResponse> {
  const response = await authFetch(`${FILES_API}/sync`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    throw new Error("Failed to sync files");
  }
  return response.json();
}

// 系统状态类型
export interface SystemStats {
  status: string;
  uptime_seconds: number;
  cpu_usage: number;
  memory: {
    total_kb: number;
    used_kb: number;
    free_kb: number;
    used_percent: number;
  };
  disk: {
    total_gb: number;
    used_gb: number;
    free_gb: number;
    used_percent: number;
  };
  network: {
    upload_bytes: number;
    download_bytes: number;
  };
}

// 获取系统状态
export async function getSystemStats(): Promise<SystemStats> {
  const response = await fetch(`${SYSTEM_API}/stats`);
  if (!response.ok) {
    throw new Error("Failed to get system stats");
  }
  return response.json();
}
