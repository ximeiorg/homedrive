import React, { createContext, useContext, useState, useEffect, type ReactNode } from "react";

export type MemberRole = "admin" | "user";

export interface Member {
  id: number;
  username: string;
  avatar: string | null;
  storage_tag: string;
  created_at: string;
  role?: MemberRole;
}

interface AuthContextType {
  isAuthenticated: boolean;
  member: Member | null;
  token: string | null;
  isAdmin: boolean;
  login: (token: string, member: Member) => void;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

// 从 localStorage 加载初始状态
function loadAuthState(): { token: string | null; member: Member | null } {
  try {
    const token = localStorage.getItem("token");
    const memberStr = localStorage.getItem("member");
    
    if (token && memberStr) {
      const member = JSON.parse(memberStr) as Member;
      return { token, member };
    }
  } catch (error) {
    console.error("Failed to load auth state:", error);
  }
  
  return { token: null, member: null };
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [authState, setAuthState] = useState<{ token: string | null; member: Member | null }>(() => {
    // 在客户端初始化时从 localStorage 加载
    if (typeof window !== "undefined") {
      return loadAuthState();
    }
    return { token: null, member: null };
  });

  useEffect(() => {
    // 同步 auth state 到 localStorage
    if (authState.token) {
      localStorage.setItem("token", authState.token);
      if (authState.member) {
        localStorage.setItem("member", JSON.stringify(authState.member));
      }
    } else {
      localStorage.removeItem("token");
      localStorage.removeItem("member");
    }
  }, [authState]);

  const login = (token: string, member: Member) => {
    setAuthState({ token, member });
  };

  const logout = () => {
    setAuthState({ token: null, member: null });
  };

  return (
    <AuthContext.Provider
      value={{
        isAuthenticated: !!authState.token,
        member: authState.member,
        token: authState.token,
        isAdmin: authState.member?.role === "admin",
        login,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}
