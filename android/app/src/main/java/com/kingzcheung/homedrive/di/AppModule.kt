package com.kingzcheung.homedrive.di

import android.content.Context
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
            level = HttpLoggingInterceptor.Level.BODY
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

    suspend fun getStaticUrl(path: String): String {
        val prefs = getPreferencesManager()
        val baseUrl = prefs.serverUrl.first().ifEmpty { "http://192.168.77.58:2300" }
        val token = prefs.token.first() ?: ""
        return if (path.startsWith("/")) {
            "$baseUrl/api/static${path}?token=$token"
        } else {
            "$baseUrl/api/static/$path?token=$token"
        }
    }
}
