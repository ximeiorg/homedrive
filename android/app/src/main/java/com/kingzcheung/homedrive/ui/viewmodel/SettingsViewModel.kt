package com.kingzcheung.homedrive.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kingzcheung.homedrive.data.local.PreferencesManager
import com.kingzcheung.homedrive.data.model.Member
import com.kingzcheung.homedrive.data.repository.AuthRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class SettingsUiState(
    val currentMember: Member? = null,
    val serverUrl: String = "",
    val isLoading: Boolean = false,
    val isUpdating: Boolean = false,
    val error: String? = null,
    val message: String? = null
)

class SettingsViewModel(
    private val authRepository: AuthRepository,
    private val preferencesManager: PreferencesManager
) : ViewModel() {

    private val _uiState = MutableStateFlow(SettingsUiState())
    val uiState: StateFlow<SettingsUiState> = _uiState.asStateFlow()

    init {
        loadMemberInfo()
        loadServerUrl()
    }

    private fun loadMemberInfo() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoading = true)

            // 从持久化数据加载会员信息
            authRepository.getMember()
                .onSuccess { member ->
                    _uiState.value = _uiState.value.copy(
                        currentMember = member,
                        isLoading = false
                    )
                }
                .onFailure {
                    _uiState.value = _uiState.value.copy(
                        isLoading = false
                    )
                }
        }
    }

    private fun loadServerUrl() {
        viewModelScope.launch {
            preferencesManager.serverUrl.collect { url ->
                _uiState.value = _uiState.value.copy(serverUrl = url)
            }
        }
    }

    fun updatePassword(oldPassword: String, newPassword: String, confirmPassword: String) {
        if (newPassword != confirmPassword) {
            _uiState.value = _uiState.value.copy(error = "新密码与确认密码不匹配")
            return
        }

        if (newPassword.length < 6) {
            _uiState.value = _uiState.value.copy(error = "密码长度至少为6位")
            return
        }

        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isUpdating = true, error = null)

            authRepository.updatePassword(oldPassword, newPassword)
                .onSuccess {
                    _uiState.value = _uiState.value.copy(
                        isUpdating = false,
                        message = "密码更新成功"
                    )
                }
                .onFailure { exception ->
                    _uiState.value = _uiState.value.copy(
                        isUpdating = false,
                        error = exception.message ?: "密码更新失败"
                    )
                }
        }
    }

    fun logout() {
        viewModelScope.launch {
            authRepository.logout()
        }
    }

    fun clearError() {
        _uiState.value = _uiState.value.copy(error = null)
    }

    fun clearMessage() {
        _uiState.value = _uiState.value.copy(message = null)
    }
}
