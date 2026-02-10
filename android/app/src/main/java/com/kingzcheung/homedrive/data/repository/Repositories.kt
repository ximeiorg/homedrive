package com.kingzcheung.homedrive.data.repository

import com.kingzcheung.homedrive.data.api.HomedriveApi
import com.kingzcheung.homedrive.data.local.PreferencesManager
import com.kingzcheung.homedrive.data.model.*
import kotlinx.coroutines.flow.first
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.MultipartBody
import okhttp3.RequestBody.Companion.asRequestBody
import okhttp3.RequestBody.Companion.toRequestBody
import java.io.File

class AuthRepository(
    private val api: HomedriveApi,
    private val preferencesManager: PreferencesManager
) {
    suspend fun login(serverUrl: String, username: String, password: String): Result<LoginResponse> {
        return try {
            val request = LoginRequest(serverUrl, username, password)
            val response = api.login(request)
            if (response.isSuccessful) {
                response.body()?.let { loginResponse ->
                    preferencesManager.setServerUrl(serverUrl)
                    preferencesManager.setToken(loginResponse.token)
                    // 保存完整的会员信息
                    loginResponse.member?.let { member ->
                        preferencesManager.setMember(member)
                    }
                    Result.success(loginResponse)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Login failed: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun logout() {
        try {
            api.logout()
        } catch (_: Exception) {
            // Ignore errors during logout
        }
        // 清除所有持久化数据
        preferencesManager.clearAll()
    }

    // 从持久化数据获取会员信息
    suspend fun getMember(): Result<Member> {
        return try {
            val member = preferencesManager.member.first()
            if (member != null) {
                Result.success(member)
            } else {
                Result.failure(Exception("No member data found"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun getCurrentUser(): Result<User> {
        return try {
            val response = api.getCurrentUser()
            if (response.isSuccessful) {
                response.body()?.let {
                    Result.success(it)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to get user: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun updatePassword(oldPassword: String, newPassword: String): Result<Unit> {
        return try {
            val map = mapOf("old_password" to oldPassword, "new_password" to newPassword)
            val response = api.updatePassword(map)
            if (response.isSuccessful) {
                Result.success(Unit)
            } else {
                Result.failure(Exception("Failed to update password: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}

class FileRepository(
    private val api: HomedriveApi,
    private val preferencesManager: PreferencesManager
) {
    suspend fun getFiles(path: String = "/", page: Int = 1): Result<PaginatedResponse<FileItem>> {
        return try {
            val response = api.getFiles(path, page)
            if (response.isSuccessful) {
                response.body()?.let { fileListResponse ->
                    Result.success(fileListResponse.toPaginatedResponse())
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to get files: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun searchFiles(query: String, type: String? = null, page: Int = 1): Result<PaginatedResponse<FileItem>> {
        return try {
            val response = api.searchFiles(query, type, page)
            if (response.isSuccessful) {
                response.body()?.let { fileListResponse ->
                    Result.success(fileListResponse.toPaginatedResponse())
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Search failed: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun getFileUrl(id: Long): Result<String> {
        return try {
            val response = api.getFileUrl(id)
            if (response.isSuccessful) {
                response.body()?.let {
                    Result.success(it["url"] ?: "")
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to get file URL: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun getServerUrl(): String {
        return ""
    }
}

class AlbumRepository(
    private val api: HomedriveApi
) {
    suspend fun getAlbums(page: Int = 1): Result<PaginatedResponse<Album>> {
        return try {
            val response = api.getAlbums(page)
            if (response.isSuccessful) {
                response.body()?.let {
                    Result.success(it)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to get albums: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun getAlbum(id: Long): Result<Album> {
        return try {
            val response = api.getAlbum(id)
            if (response.isSuccessful) {
                response.body()?.let {
                    Result.success(it)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to get album: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun getAlbumFiles(id: Long, page: Int = 1): Result<PaginatedResponse<FileItem>> {
        return try {
            val response = api.getAlbumFiles(id, page)
            if (response.isSuccessful) {
                response.body()?.let {
                    Result.success(it)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to get album files: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}

class ShareRepository(
    private val api: HomedriveApi
) {
    suspend fun getShares(page: Int = 1): Result<PaginatedResponse<Share>> {
        return try {
            val response = api.getShares(page)
            if (response.isSuccessful) {
                response.body()?.let {
                    Result.success(it)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to get shares: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun createShare(fileId: Long, userIds: List<Long>, expiresAt: String? = null): Result<Share> {
        return try {
            val map = mutableMapOf<String, Any>("user_ids" to userIds)
            expiresAt?.let { map["expires_at"] = it }
            val response = api.createShare(fileId, map)
            if (response.isSuccessful) {
                response.body()?.let {
                    Result.success(it)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to create share: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun getShareUsers(): Result<List<ShareUser>> {
        return try {
            val response = api.getShareUsers()
            if (response.isSuccessful) {
                response.body()?.let {
                    Result.success(it)
                } ?: Result.success(emptyList())
            } else {
                Result.failure(Exception("Failed to get share users: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun deleteShare(id: Long): Result<Unit> {
        return try {
            val response = api.deleteShare(id)
            if (response.isSuccessful) {
                Result.success(Unit)
            } else {
                Result.failure(Exception("Failed to delete share: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}

class UploadRepository(
    private val api: HomedriveApi
) {
    suspend fun uploadFile(file: File, path: String = "/"): Result<UploadResponse> {
        return try {
            val mimeType = file.extension.toMimeType() ?: "application/octet-stream"
            val requestBody = file.asRequestBody(mimeType.toMediaTypeOrNull())
            val multipart = MultipartBody.Part.createFormData("file", file.name, requestBody)

            val response = api.uploadFile(multipart)
            if (response.isSuccessful) {
                response.body()?.let {
                    Result.success(it)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                val errorBody = response.errorBody()?.string() ?: "Unknown error"
                Result.failure(Exception("Upload failed: ${response.code()} - $errorBody"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    private fun String.toMimeType(): String? {
        return when (this.lowercase()) {
            "jpg", "jpeg" -> "image/jpeg"
            "png" -> "image/png"
            "gif" -> "image/gif"
            "webp" -> "image/webp"
            "mp4" -> "video/mp4"
            "mov" -> "video/quicktime"
            "avi" -> "video/x-msvideo"
            "mkv" -> "video/x-matroska"
            else -> "application/octet-stream"
        }
    }
}
