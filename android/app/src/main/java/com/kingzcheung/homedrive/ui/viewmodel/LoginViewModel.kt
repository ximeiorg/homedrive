package com.kingzcheung.homedrive.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kingzcheung.homedrive.data.model.Member
import com.kingzcheung.homedrive.data.network.DiscoveredServer
import com.kingzcheung.homedrive.data.network.NetworkScanner
import com.kingzcheung.homedrive.data.repository.AuthRepository
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class LoginUiState(
    val serverUrl: String = "",
    val username: String = "",
    val password: String = "",
    val isLoading: Boolean = false,
    val error: String? = null,
    val isLoggedIn: Boolean = false,
    val currentMember: Member? = null,
    // 网络扫描相关状态
    val isScanning: Boolean = false,
    val discoveredServers: List<DiscoveredServer> = emptyList(),
    val scanProgress: Int = 0,
    val scanTotal: Int = 254,
    val showServerDropdown: Boolean = false,
    val isManualInput: Boolean = false
)

class LoginViewModel(
    private val authRepository: AuthRepository,
    private val networkScanner: NetworkScanner
) : ViewModel() {

    private val _uiState = MutableStateFlow(LoginUiState())
    val uiState: StateFlow<LoginUiState> = _uiState.asStateFlow()
    
    private var scanJob: Job? = null

    init {
        // 初始化时自动开始扫描
        startNetworkScan()
    }
    
    fun startNetworkScan() {
        scanJob?.cancel()
        scanJob = viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isScanning = true,
                scanProgress = 0,
                discoveredServers = emptyList(),
                error = null
            )
            
            val servers = networkScanner.scanNetwork(
                port = 2300,
                onProgress = { scanned, total ->
                    _uiState.value = _uiState.value.copy(
                        scanProgress = scanned,
                        scanTotal = total
                    )
                }
            )
            
            _uiState.value = _uiState.value.copy(
                isScanning = false,
                discoveredServers = servers,
                // 如果找到服务器，默认选择第一个；否则切换到手动输入模式
                isManualInput = servers.isEmpty(),
                serverUrl = if (servers.isNotEmpty()) servers.first().fullUrl else ""
            )
        }
    }
    
    fun selectServer(server: DiscoveredServer) {
        _uiState.value = _uiState.value.copy(
            serverUrl = server.fullUrl,
            showServerDropdown = false,
            isManualInput = false,
            error = null
        )
    }
    
    fun selectManualInput() {
        _uiState.value = _uiState.value.copy(
            isManualInput = true,
            showServerDropdown = false,
            serverUrl = "",
            error = null
        )
    }
    
    fun toggleServerDropdown(show: Boolean) {
        _uiState.value = _uiState.value.copy(showServerDropdown = show)
    }

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
    
    override fun onCleared() {
        super.onCleared()
        scanJob?.cancel()
    }
}
