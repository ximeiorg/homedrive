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

    // ============ 相册 ============

    @GET("members/{memberId}/albums")
    suspend fun getAlbums(
        @Path("memberId") memberId: Long,
        @Query("page") page: Int = 1,
        @Query("page_size") pageSize: Int = DEFAULT_PAGE_SIZE
    ): Response<AlbumListResponse>

    @GET("members/{memberId}/albums/{albumId}")
    suspend fun getAlbum(
        @Path("memberId") memberId: Long,
        @Path("albumId") albumId: Long
    ): Response<AlbumResponse>

    @POST("members/{memberId}/albums")
    suspend fun createAlbum(
        @Path("memberId") memberId: Long,
        @Body request: CreateAlbumRequest
    ): Response<AlbumResponse>

    @PUT("members/{memberId}/albums/{albumId}")
    suspend fun updateAlbum(
        @Path("memberId") memberId: Long,
        @Path("albumId") albumId: Long,
        @Body request: UpdateAlbumRequest
    ): Response<AlbumResponse>

    @DELETE("members/{memberId}/albums/{albumId}")
    suspend fun deleteAlbum(
        @Path("memberId") memberId: Long,
        @Path("albumId") albumId: Long
    ): Response<Unit>

    @GET("members/{memberId}/albums/{albumId}/files")
    suspend fun getAlbumFiles(
        @Path("memberId") memberId: Long,
        @Path("albumId") albumId: Long,
        @Query("page") page: Int = 1,
        @Query("page_size") pageSize: Int = DEFAULT_PAGE_SIZE
    ): Response<AlbumFilesResponse>

    @POST("members/{memberId}/albums/{albumId}/files")
    suspend fun addFilesToAlbum(
        @Path("memberId") memberId: Long,
        @Path("albumId") albumId: Long,
        @Body request: AddFilesRequest
    ): Response<AddFilesResponse>

    @HTTP(method = "DELETE", path = "members/{memberId}/albums/{albumId}/files", hasBody = true)
    suspend fun removeFilesFromAlbum(
        @Path("memberId") memberId: Long,
        @Path("albumId") albumId: Long,
        @Body request: RemoveFilesRequest
    ): Response<RemoveFilesResponse>

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
