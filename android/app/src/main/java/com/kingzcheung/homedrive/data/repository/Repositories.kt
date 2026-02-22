package com.kingzcheung.homedrive.data.repository

import com.kingzcheung.homedrive.data.api.HomedriveApi
import com.kingzcheung.homedrive.data.local.PreferencesManager
import com.kingzcheung.homedrive.data.model.*
import com.kingzcheung.homedrive.di.AppContainer
import kotlinx.coroutines.flow.first
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.MultipartBody
import okhttp3.OkHttpClient
import okhttp3.RequestBody.Companion.asRequestBody
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.logging.HttpLoggingInterceptor
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory
import java.io.File
import java.util.concurrent.TimeUnit

class AuthRepository(
    private val preferencesManager: PreferencesManager
) {
    /**
     * 创建临时 API 实例用于登录
     * 登录时需要使用用户指定的服务器地址
     */
    private fun createApi(serverUrl: String): HomedriveApi {
        val loggingInterceptor = HttpLoggingInterceptor().apply {
            level = HttpLoggingInterceptor.Level.HEADERS
        }

        val client = OkHttpClient.Builder()
            .addInterceptor(loggingInterceptor)
            .connectTimeout(30, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .writeTimeout(60, TimeUnit.SECONDS)
            .build()

        val retrofit = Retrofit.Builder()
            .baseUrl("$serverUrl/api/")
            .client(client)
            .addConverterFactory(GsonConverterFactory.create())
            .build()

        return retrofit.create(HomedriveApi::class.java)
    }
    
    /**
     * 获取已保存的服务器地址对应的 API 实例
     */
    private suspend fun getSavedApi(): HomedriveApi? {
        val serverUrl = preferencesManager.serverUrl.first()
        return if (serverUrl.isNotEmpty()) {
            createApi(serverUrl)
        } else {
            null
        }
    }
    
    suspend fun login(serverUrl: String, username: String, password: String): Result<LoginResponse> {
        return try {
            // 使用用户指定的服务器地址创建 API
            val api = createApi(serverUrl)
            
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
                val errorBody = response.errorBody()?.string() ?: "Unknown error"
                Result.failure(Exception("登录失败: ${response.code()} - $errorBody"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun logout() {
        try {
            getSavedApi()?.logout()
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
            val api = getSavedApi() ?: return Result.failure(Exception("未登录或服务器地址未设置"))
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
            val api = getSavedApi() ?: return Result.failure(Exception("未登录或服务器地址未设置"))
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
    private val api: HomedriveApi,
    private val preferencesManager: PreferencesManager
) {
    private suspend fun getMemberId(): Long {
        val member = preferencesManager.member.first()
        return member?.id ?: throw Exception("User not logged in")
    }

    suspend fun getAlbums(page: Int = 1): Result<PaginatedResponse<Album>> {
        return try {
            val memberId = getMemberId()
            val response = api.getAlbums(memberId, page)
            if (response.isSuccessful) {
                response.body()?.let {
                    Result.success(it.toPaginatedResponse())
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to get albums: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun getAlbum(id: Long): Result<AlbumResponse> {
        return try {
            val memberId = getMemberId()
            val response = api.getAlbum(memberId, id)
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

    suspend fun createAlbum(request: CreateAlbumRequest): Result<AlbumResponse> {
        return try {
            val memberId = getMemberId()
            val response = api.createAlbum(memberId, request)
            if (response.isSuccessful) {
                response.body()?.let {
                    Result.success(it)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to create album: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun updateAlbum(albumId: Long, request: UpdateAlbumRequest): Result<AlbumResponse> {
        return try {
            val memberId = getMemberId()
            val response = api.updateAlbum(memberId, albumId, request)
            if (response.isSuccessful) {
                response.body()?.let {
                    Result.success(it)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to update album: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun deleteAlbum(albumId: Long): Result<Unit> {
        return try {
            val memberId = getMemberId()
            val response = api.deleteAlbum(memberId, albumId)
            if (response.isSuccessful) {
                Result.success(Unit)
            } else {
                Result.failure(Exception("Failed to delete album: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun getAlbumFiles(albumId: Long, page: Int = 1): Result<PaginatedResponse<FileItem>> {
        return try {
            val memberId = getMemberId()
            val response = api.getAlbumFiles(memberId, albumId, page)
            if (response.isSuccessful) {
                response.body()?.let {
                    Result.success(it.toPaginatedResponse())
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to get album files: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun addFilesToAlbum(albumId: Long, fileIds: List<Long>): Result<AddFilesResponse> {
        return try {
            val memberId = getMemberId()
            val response = api.addFilesToAlbum(memberId, albumId, AddFilesRequest(fileIds))
            if (response.isSuccessful) {
                response.body()?.let {
                    Result.success(it)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to add files: ${response.message()}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun removeFilesFromAlbum(albumId: Long, fileIds: List<Long>): Result<RemoveFilesResponse> {
        return try {
            val memberId = getMemberId()
            val response = api.removeFilesFromAlbum(memberId, albumId, RemoveFilesRequest(fileIds))
            if (response.isSuccessful) {
                response.body()?.let {
                    Result.success(it)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception("Failed to remove files: ${response.message()}"))
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
