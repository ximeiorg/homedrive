package com.kingzcheung.homedrive.di

import com.kingzcheung.homedrive.data.local.PreferencesManager
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import okhttp3.Interceptor
import okhttp3.Response

class AuthInterceptor(
    private val preferencesManager: PreferencesManager
) : Interceptor {

    companion object {
        // 登出回调，当收到 401 时触发
        var onUnauthorized: (() -> Unit)? = null
    }

    override fun intercept(chain: Interceptor.Chain): Response {
        val originalRequest = chain.request()

        val token = runBlocking {
            preferencesManager.token.first()
        }

        if (token.isNullOrEmpty()) {
            return chain.proceed(originalRequest)
        }

        val newRequest = originalRequest.newBuilder()
            .header("Authorization", "Bearer $token")
            .build()

        val response = chain.proceed(newRequest)

        // 检查是否为 401 未授权响应
        if (response.code == 401) {
            // 清除本地存储的认证信息
            runBlocking {
                preferencesManager.clearAll()
            }
            // 通知应用需要登出
            onUnauthorized?.invoke()
        }

        return response
    }
}
