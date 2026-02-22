package com.kingzcheung.homedrive

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import kotlinx.coroutines.flow.first
import com.kingzcheung.homedrive.di.AppContainer
import com.kingzcheung.homedrive.di.AuthInterceptor
import com.kingzcheung.homedrive.ui.screen.HomeScreen
import com.kingzcheung.homedrive.ui.screen.LoginScreen
import com.kingzcheung.homedrive.ui.theme.HomedriveTheme
import com.kingzcheung.homedrive.ui.viewmodel.*

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        val preferencesManager = AppContainer.getPreferencesManager()

        setContent {
            HomedriveTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    var isLoggedIn by remember { mutableStateOf<Boolean?>(null) }
                    var api by remember { mutableStateOf<com.kingzcheung.homedrive.data.api.HomedriveApi?>(null) }

                    // 设置 401 未授权回调，自动登出
                    DisposableEffect(Unit) {
                        AuthInterceptor.onUnauthorized = {
                            isLoggedIn = false
                            api = null
                        }
                        onDispose {
                            AuthInterceptor.onUnauthorized = null
                        }
                    }

                    LaunchedEffect(Unit) {
                        // 检查是否已登录（通过检查 token 和服务器地址是否存在）
                        val token = preferencesManager.token.first()
                        val serverUrl = preferencesManager.serverUrl.first()
                        isLoggedIn = token?.isNotEmpty() == true && serverUrl.isNotEmpty()
                        
                        // 如果已登录，创建 API 实例
                        if (isLoggedIn == true) {
                            try {
                                api = AppContainer.createApi()
                            } catch (e: Exception) {
                                isLoggedIn = false
                            }
                        }
                    }

                    when (isLoggedIn) {
                        null -> { }
                        true -> {
                            // 已登录，api 应该已经创建
                            val currentApi = api
                            if (currentApi != null) {
                                val fileRepository = remember { com.kingzcheung.homedrive.data.repository.FileRepository(currentApi, preferencesManager) }
                                val galleryViewModel = remember { GalleryViewModel(fileRepository) }
                                val uploadViewModel = remember { UploadViewModel(currentApi, preferencesManager) }
                                val albumViewModel = remember { AlbumViewModel(currentApi, preferencesManager) }
                                val shareViewModel = remember { ShareViewModel(currentApi) }
                                val authRepository = remember { com.kingzcheung.homedrive.data.repository.AuthRepository(preferencesManager) }
                                val settingsViewModel = remember { SettingsViewModel(authRepository, preferencesManager) }
                                val mediaViewerViewModel = remember { MediaViewerViewModel() }
                                
                                HomeScreen(
                                    onLogout = {
                                        isLoggedIn = false
                                        api = null
                                    },
                                    galleryViewModel = galleryViewModel,
                                    uploadViewModel = uploadViewModel,
                                    albumViewModel = albumViewModel,
                                    shareViewModel = shareViewModel,
                                    settingsViewModel = settingsViewModel,
                                    mediaViewerViewModel = mediaViewerViewModel
                                )
                            }
                        }
                        false -> {
                            val authRepository = remember { com.kingzcheung.homedrive.data.repository.AuthRepository(preferencesManager) }
                            val loginViewModel = remember { LoginViewModel(authRepository, AppContainer.getNetworkScanner()) }
                            
                            LoginScreen(
                                onLoginSuccess = {
                                    // 登录成功后创建 API 实例
                                    try {
                                        api = AppContainer.createApi()
                                        isLoggedIn = true
                                    } catch (e: Exception) {
                                        // 如果创建 API 失败，保持在登录页面
                                    }
                                },
                                viewModel = loginViewModel
                            )
                        }
                    }
                }
            }
        }
    }
}
