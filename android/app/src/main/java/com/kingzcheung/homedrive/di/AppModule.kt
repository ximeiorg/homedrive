package com.kingzcheung.homedrive.di

import android.content.Context
import android.util.Log
import com.kingzcheung.homedrive.data.api.HomedriveApi
import com.kingzcheung.homedrive.data.local.PreferencesManager
import com.kingzcheung.homedrive.data.network.NetworkScanner
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import okhttp3.OkHttpClient
import okhttp3.logging.HttpLoggingInterceptor
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory
import java.util.concurrent.TimeUnit

/**
 * Simple dependency injection container without Hilt
 */
object AppContainer {
    private const val TAG = "AppContainer"
    
    private lateinit var applicationContext: Context
    private var preferencesManager: PreferencesManager? = null
    private var networkScanner: NetworkScanner? = null

    fun initialize(context: Context) {
        applicationContext = context.applicationContext
    }

    fun getPreferencesManager(): PreferencesManager {
        if (preferencesManager == null) {
            preferencesManager = PreferencesManager(applicationContext)
        }
        return preferencesManager!!
    }

    /**
     * 获取已保存的服务器地址
     */
    suspend fun getBaseUrl(): String {
        val prefs = getPreferencesManager()
        return prefs.serverUrl.first()
    }

    /**
     * 创建带认证拦截器的 OkHttpClient
     */
    fun createOkHttpClient(): OkHttpClient {
        val preferencesManager = getPreferencesManager()
        val loggingInterceptor = HttpLoggingInterceptor().apply {
            level = HttpLoggingInterceptor.Level.HEADERS
        }

        return OkHttpClient.Builder()
            .addInterceptor(AuthInterceptor(preferencesManager))
            .addInterceptor(loggingInterceptor)
            .connectTimeout(30, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .writeTimeout(60, TimeUnit.SECONDS)
            .build()
    }

    /**
     * 创建 API 实例
     * 必须在登录后调用，此时服务器地址已保存
     */
    fun createApi(): HomedriveApi {
        val baseUrl = runBlocking { getBaseUrl() }
        if (baseUrl.isEmpty()) {
            throw IllegalStateException("服务器地址未设置，请先登录")
        }
        
        val retrofit = Retrofit.Builder()
            .baseUrl("$baseUrl/api/")
            .client(createOkHttpClient())
            .addConverterFactory(GsonConverterFactory.create())
            .build()

        return retrofit.create(HomedriveApi::class.java)
    }

    /**
     * 获取带认证 token 的静态资源 URL
     * @param urlOrPath 服务器返回的 URL（可能是完整 URL 或相对路径）
     * @return 带有 token 参数的完整 URL，使用用户配置的服务器地址
     */
    suspend fun getStaticUrl(urlOrPath: String): String {
        val prefs = getPreferencesManager()
        val token = prefs.token.first() ?: ""
        val baseUrl = prefs.serverUrl.first()
        
        Log.d(TAG, "getStaticUrl input: $urlOrPath, baseUrl: $baseUrl")
        
        if (baseUrl.isEmpty()) {
            Log.w(TAG, "getStaticUrl: No server URL configured")
            return urlOrPath
        }
        
        // 提取路径部分（去掉服务器地址）
        val path = when {
            // 完整 URL，提取路径部分
            urlOrPath.startsWith("http://") || urlOrPath.startsWith("https://") -> {
                // 从 http://xxx:port 后面提取路径
                val afterProtocol = urlOrPath.substringAfter("://")
                val pathStart = afterProtocol.indexOf('/')
                if (pathStart >= 0) {
                    afterProtocol.substring(pathStart)
                } else {
                    "/$urlOrPath"
                }
            }
            // 相对路径，直接使用
            urlOrPath.startsWith("/") -> urlOrPath
            else -> "/$urlOrPath"
        }
        
        // 使用用户配置的服务器地址构建完整 URL
        val fullUrl = if (path.contains("?")) {
            "$baseUrl$path&token=$token"
        } else {
            "$baseUrl$path?token=$token"
        }
        
        Log.d(TAG, "getStaticUrl result: $fullUrl")
        return fullUrl
    }
    
    fun getNetworkScanner(): NetworkScanner {
        if (networkScanner == null) {
            networkScanner = NetworkScanner(applicationContext)
        }
        return networkScanner!!
    }
}
