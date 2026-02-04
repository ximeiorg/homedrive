const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || "http://127.0.0.1:2300";
const API_BASE = `${API_BASE_URL}/api/members`;

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

// 检查 member 表是否为空
export async function checkMembersEmpty(): Promise<IsEmptyResponse> {
  const response = await fetch(`${API_BASE}/empty`);
  if (!response.ok) {
    throw new Error("Failed to check members empty status");
  }
  return response.json();
}

// 初始化管理员
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

// 登录
export async function login(data: LoginRequest): Promise<LoginResponse> {
  const response = await fetch(`${API_BASE}/login`, {
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
