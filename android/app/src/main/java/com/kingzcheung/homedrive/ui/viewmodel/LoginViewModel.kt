package com.kingzcheung.homedrive.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kingzcheung.homedrive.data.local.PreferencesManager
import com.kingzcheung.homedrive.data.model.LoginResponse
import com.kingzcheung.homedrive.data.model.Member
import com.kingzcheung.homedrive.data.repository.AuthRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class LoginUiState(
    val serverUrl: String = "http://192.168.77.58:2300",
    val username: String = "",
    val password: String = "",
    val isLoading: Boolean = false,
    val error: String? = null,
    val isLoggedIn: Boolean = false,
    val currentMember: Member? = null
)

class LoginViewModel(
    private val authRepository: AuthRepository
) : ViewModel() {

    private val _uiState = MutableStateFlow(LoginUiState())
    val uiState: StateFlow<LoginUiState> = _uiState.asStateFlow()

    fun updateServerUrl(url: String) {
        _uiState.value = _uiState.value.copy(serverUrl = url, error = null)
    }

    fun updateUsername(username: String) {
        _uiState.value = _uiState.value.copy(username = username, error = null)
    }

    fun updatePassword(password: String) {
        _uiState.value = _uiState.value.copy(password = password, error = null)
    }

    fun login() {
        val currentState = _uiState.value

        if (currentState.serverUrl.isBlank()) {
            _uiState.value = currentState.copy(error = "请输入服务器地址")
            return
        }

        if (currentState.username.isBlank()) {
            _uiState.value = currentState.copy(error = "请输入用户名")
            return
        }

        if (currentState.password.isBlank()) {
            _uiState.value = currentState.copy(error = "请输入密码")
            return
        }

        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoading = true, error = null)

            authRepository.login(
                serverUrl = currentState.serverUrl,
                username = currentState.username,
                password = currentState.password
            ).onSuccess { loginResponse ->
                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    isLoggedIn = true,
                    currentMember = loginResponse.member
                )
            }.onFailure { exception ->
                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    error = exception.message ?: "登录失败"
                )
            }
        }
    }

    fun clearError() {
        _uiState.value = _uiState.value.copy(error = null)
    }
}
