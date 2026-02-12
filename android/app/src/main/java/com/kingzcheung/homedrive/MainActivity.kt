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
        val api = AppContainer.getApi()

        // Create ViewModels
        val galleryViewModel = GalleryViewModel()
        val uploadViewModel = UploadViewModel(api, preferencesManager)
        val albumViewModel = AlbumViewModel(api, preferencesManager)
        val shareViewModel = ShareViewModel(api)
        val settingsViewModel = SettingsViewModel(
            authRepository = com.kingzcheung.homedrive.data.repository.AuthRepository(api, preferencesManager),
            preferencesManager = preferencesManager
        )
        val loginViewModel = LoginViewModel(
            authRepository = com.kingzcheung.homedrive.data.repository.AuthRepository(api, preferencesManager)
        )
        val mediaViewerViewModel = MediaViewerViewModel()

        setContent {
            HomedriveTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    var isLoggedIn by remember { mutableStateOf<Boolean?>(null) }

                    // 设置 401 未授权回调，自动登出
                    DisposableEffect(Unit) {
                        AuthInterceptor.onUnauthorized = {
                            isLoggedIn = false
                        }
                        onDispose {
                            AuthInterceptor.onUnauthorized = null
                        }
                    }

                    LaunchedEffect(Unit) {
                        // 检查是否已登录（通过检查 token 是否存在）
                        val token = preferencesManager.token.first()
                        isLoggedIn = token?.isNotEmpty() == true
                    }

                    when (isLoggedIn) {
                        null -> { }
                        true -> {
                            HomeScreen(
                                onLogout = {
                                    isLoggedIn = false
                                },
                                galleryViewModel = galleryViewModel,
                                uploadViewModel = uploadViewModel,
                                albumViewModel = albumViewModel,
                                shareViewModel = shareViewModel,
                                settingsViewModel = settingsViewModel,
                                mediaViewerViewModel = mediaViewerViewModel
                            )
                        }
                        false -> {
                            LoginScreen(
                                onLoginSuccess = {
                                    isLoggedIn = true
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
