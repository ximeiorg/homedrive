# ========== 多阶段构建 ==========

# 阶段 1: 构建前端
FROM node:20-alpine AS frontend-builder

# 安装 pnpm
RUN corepack enable && corepack prepare pnpm@9 --activate

WORKDIR /app/web

# 复制前端代码
COPY web/package.json web/pnpm-lock.yaml* ./

# 安装依赖
RUN pnpm install --frozen-lockfile

# 复制前端源码
COPY web/ ./

# 构建前端
RUN pnpm build

# ========== 阶段 2: 构建后端 (静态链接 musl)
FROM rust:1.88-bookworm AS backend-builder

# 安装 musl-tools 用于静态链接
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*

# 设置目标架构
ENV TARGET=x86_64-unknown-linux-musl
RUN rustup target add $TARGET

WORKDIR /app

# 复制 Cargo 配置
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./
COPY default.toml ./

# 复制 crates
COPY crates/ ./crates/

# 复制前端构建资源 (供 rust-embed 使用)
COPY --from=frontend-builder /app/web/build/client ./web/build/client

# 构建后端 (静态链接)
RUN cargo build --release --target $TARGET --bin homedrive

# ========== 阶段 4: 运行镜像
FROM alpine:3.20 AS runner

# 安装运行时依赖
RUN apk add --no-cache ca-certificates

# 创建非 root 用户
RUN addgroup -g 1000 homedrive && adduser -u 1000 -G homedrive -s /bin/sh -D homedrive

WORKDIR /app

# 从构建阶段复制前端资源
COPY --from=frontend-builder /app/web/build/client ./public

# 从构建阶段复制后端二进制
COPY --from=backend-builder /app/target/x86_64-unknown-linux-musl/release/homedrive ./homedrive

# 创建存储目录
RUN mkdir -p /app/storage && chown -R homedrive:homedrive /app

USER homedrive

# 暴露端口
EXPOSE 2300

# 启动命令
CMD ["./homedrive"]

