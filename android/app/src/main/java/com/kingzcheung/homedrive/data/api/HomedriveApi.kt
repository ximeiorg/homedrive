package com.kingzcheung.homedrive.data.api

import com.kingzcheung.homedrive.data.model.*
import okhttp3.MultipartBody
import retrofit2.Response
import retrofit2.http.*

interface HomedriveApi {

    companion object {
        const val DEFAULT_PAGE_SIZE = 50
    }

    // ============ 认证 ============

    @POST("auth/login")
    suspend fun login(@Body request: LoginRequest): Response<LoginResponse>

    @POST("auth/logout")
    suspend fun logout(): Response<Unit>

    @GET("auth/me")
    suspend fun getCurrentUser(): Response<User>

    @PUT("auth/password")
    suspend fun updatePassword(
        @Body map: Map<String, String>
    ): Response<Unit>

    // ============ 文件 ============

    @GET("files")
    suspend fun getFiles(
        @Query("path") path: String = "/",
        @Query("page") page: Int = 1,
        @Query("page_size") pageSize: Int = DEFAULT_PAGE_SIZE
    ): Response<FileListResponse>

    @GET("files/{id}")
    suspend fun getFile(@Path("id") id: Long): Response<FileItem>

    @GET("files/{id}/url")
    suspend fun getFileUrl(@Path("id") id: Long): Response<Map<String, String>>

    @GET("files/search")
    suspend fun searchFiles(
        @Query("q") query: String,
        @Query("type") type: String? = null,
        @Query("page") page: Int = 1,
        @Query("page_size") pageSize: Int = DEFAULT_PAGE_SIZE
    ): Response<FileListResponse>

    // ============ 图集 ============

    @GET("albums")
    suspend fun getAlbums(
        @Query("page") page: Int = 1,
        @Query("page_size") pageSize: Int = DEFAULT_PAGE_SIZE
    ): Response<PaginatedResponse<Album>>

    @GET("albums/{id}")
    suspend fun getAlbum(@Path("id") id: Long): Response<Album>

    @GET("albums/{id}/files")
    suspend fun getAlbumFiles(
        @Path("id") id: Long,
        @Query("page") page: Int = 1,
        @Query("page_size") pageSize: Int = DEFAULT_PAGE_SIZE
    ): Response<PaginatedResponse<FileItem>>

    // ============ 分享 ============

    @GET("shares")
    suspend fun getShares(
        @Query("page") page: Int = 1,
        @Query("page_size") pageSize: Int = DEFAULT_PAGE_SIZE
    ): Response<PaginatedResponse<Share>>

    @POST("files/{id}/share")
    suspend fun createShare(
        @Path("id") id: Long,
        @Body map: Map<String, Any>
    ): Response<Share>

    @GET("shares/users")
    suspend fun getShareUsers(): Response<List<ShareUser>>

    @DELETE("shares/{id}")
    suspend fun deleteShare(@Path("id") id: Long): Response<Unit>

    // ============ 上传 ============

    @Multipart
    @POST("files/upload")
    suspend fun uploadFile(
        @Part file: MultipartBody.Part
    ): Response<UploadResponse>
}
