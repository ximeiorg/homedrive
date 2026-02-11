package com.kingzcheung.homedrive.di

import android.content.Context
import android.util.Log
import com.kingzcheung.homedrive.data.api.HomedriveApi
import com.kingzcheung.homedrive.data.local.PreferencesManager
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
    private var api: HomedriveApi? = null

    fun initialize(context: Context) {
        applicationContext = context.applicationContext
    }

    fun getPreferencesManager(): PreferencesManager {
        if (preferencesManager == null) {
            preferencesManager = PreferencesManager(applicationContext)
        }
        return preferencesManager!!
    }

    fun getBaseUrl(): String {
        return runBlocking {
            val prefs = getPreferencesManager()
            prefs.serverUrl.first().ifEmpty { "http://192.168.77.58:2300" }
        }
    }

    fun getOkHttpClient(): OkHttpClient {
        val preferencesManager = getPreferencesManager()
        val loggingInterceptor = HttpLoggingInterceptor().apply {
            // 使用 HEADERS 级别避免输出大量文件内容（图片等二进制数据）
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

    fun getRetrofit(): Retrofit {
        return Retrofit.Builder()
            .baseUrl("${getBaseUrl()}/api/")
            .client(getOkHttpClient())
            .addConverterFactory(GsonConverterFactory.create())
            .build()
    }

    fun getApi(): HomedriveApi {
        if (api == null) {
            api = getRetrofit().create(HomedriveApi::class.java)
        }
        return api!!
    }

    /**
     * 获取带认证 token 的静态资源 URL
     * @param urlOrPath 服务器返回的 URL（可能是完整 URL 或相对路径）
     * @return 带有 token 参数的完整 URL
     */
    suspend fun getStaticUrl(urlOrPath: String): String {
        val prefs = getPreferencesManager()
        val token = prefs.token.first() ?: ""
        
        Log.d(TAG, "getStaticUrl input: $urlOrPath")
        
        // 如果已经是完整的 URL，直接添加 token
        val result = if (urlOrPath.startsWith("http://") || urlOrPath.startsWith("https://")) {
            // 已经是完整 URL，检查是否已有查询参数
            if (urlOrPath.contains("?")) {
                "$urlOrPath&token=$token"
            } else {
                "$urlOrPath?token=$token"
            }
        } else {
            // 相对路径，需要拼接 baseUrl
            val baseUrl = prefs.serverUrl.first().ifEmpty { "http://192.168.77.58:2300" }
            val path = if (urlOrPath.startsWith("/")) {
                urlOrPath
            } else {
                "/$urlOrPath"
            }
            "$baseUrl/api/static$path?token=$token"
        }
        
        Log.d(TAG, "getStaticUrl result: $result")
        return result
    }
}
