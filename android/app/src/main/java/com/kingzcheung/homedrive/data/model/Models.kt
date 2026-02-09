package com.kingzcheung.homedrive.data.model

import com.google.gson.annotations.SerializedName

// 登录请求
data class LoginRequest(
    @SerializedName("server_url") val serverUrl: String,
    @SerializedName("username") val username: String,
    @SerializedName("password") val password: String
)

// 登录响应
data class LoginResponse(
    @SerializedName("token") val token: String,
    @SerializedName("member") val member: User?
)

// 用户模型
data class User(
    @SerializedName("id") val id: Long,
    @SerializedName("username") val username: String,
    @SerializedName("email") val email: String? = null,
    @SerializedName("avatar") val avatar: String? = null,
    @SerializedName("storage_tag") val storageTag: String? = null,
    @SerializedName("created_at") val createdAt: String? = null
)

// 文件模型
data class FileItem(
    @SerializedName("id") val id: Long,
    @SerializedName("file_name") val name: String,
    @SerializedName("description") val description: String? = null,
    @SerializedName("file_size") val size: Long? = null,
    @SerializedName("mime_type") val mimeType: String? = null,
    @SerializedName("thumbnail") val thumbnail: String? = null,
    @SerializedName("url") val url: String? = null,
    @SerializedName("created_at") val createdAt: String,
    @SerializedName("updated_at") val updatedAt: String
) {
    val path: String
        get() = description ?: ""

    val type: FileType
        get() = when {
            mimeType?.startsWith("image/") == true -> FileType.IMAGE
            mimeType?.startsWith("video/") == true -> FileType.VIDEO
            else -> FileType.FILE
        }
}

enum class FileType {
    @SerializedName("file") FILE,
    @SerializedName("folder") FOLDER,
    @SerializedName("image") IMAGE,
    @SerializedName("video") VIDEO
}

// 图集模型
data class Album(
    @SerializedName("id") val id: Long,
    @SerializedName("name") val name: String,
    @SerializedName("cover") val cover: String? = null,
    @SerializedName("count") val count: Int = 0,
    @SerializedName("created_at") val createdAt: String,
    @SerializedName("updated_at") val updatedAt: String
)

// 分享模型
data class Share(
    @SerializedName("id") val id: Long,
    @SerializedName("file_id") val fileId: Long,
    @SerializedName("share_link") val shareLink: String,
    @SerializedName("expires_at") val expiresAt: String? = null,
    @SerializedName("permissions") val permissions: List<String>,
    @SerializedName("created_at") val createdAt: String
)

// 分享用户
data class ShareUser(
    @SerializedName("id") val id: Long,
    @SerializedName("username") val username: String,
    @SerializedName("email") val email: String? = null
)

// 分页响应
data class PaginatedResponse<T>(
    @SerializedName("data") val data: List<T>,
    @SerializedName("page") val page: Int,
    @SerializedName("page_size") val pageSize: Int,
    @SerializedName("total") val total: Long,
    @SerializedName("has_more") val hasMore: Boolean
)

// 文件列表响应（服务器端返回格式）
data class FileListResponse(
    @SerializedName("files") val files: List<FileItem>,
    @SerializedName("total") val total: Long,
    @SerializedName("page") val page: Int,
    @SerializedName("page_size") val pageSize: Int,
    @SerializedName("total_pages") val totalPages: Int
) {
    fun toPaginatedResponse(): PaginatedResponse<FileItem> {
        return PaginatedResponse(
            data = files,
            page = page,
            pageSize = pageSize,
            total = total,
            hasMore = page < totalPages
        )
    }
}

// 上传响应
data class UploadResponse(
    @SerializedName("id") val id: Long,
    @SerializedName("name") val name: String,
    @SerializedName("path") val path: String,
    @SerializedName("thumbnail") val thumbnail: String? = null
)

// API 错误响应
data class ApiError(
    @SerializedName("error") val error: String,
    @SerializedName("message") val message: String
)
