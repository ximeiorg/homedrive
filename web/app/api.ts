const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || "http://127.0.0.1:2300";
const API_BASE = `${API_BASE_URL}/api`;
const MEMBERS_API = `${API_BASE_URL}/api/members`;
const FILES_API = `${API_BASE_URL}/api/files`;
const AUTH_API = `${API_BASE_URL}/api/auth`;

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
async function authFetch(
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
export async function getFileList(): Promise<{
  files: Array<{
    id: number;
    file_name: string;
    description: string;
    created_at: string;
    updated_at: string;
    url: string | null;
  }>;
}> {
  const response = await authFetch(FILES_API);
  if (!response.ok) {
    throw new Error("Failed to get file list");
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
