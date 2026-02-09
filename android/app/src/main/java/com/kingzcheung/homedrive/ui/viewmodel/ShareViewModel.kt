package com.kingzcheung.homedrive.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kingzcheung.homedrive.data.api.HomedriveApi
import com.kingzcheung.homedrive.data.model.Share
import com.kingzcheung.homedrive.data.model.ShareUser
import com.kingzcheung.homedrive.data.repository.ShareRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class ShareUiState(
    val shares: List<Share> = emptyList(),
    val shareUsers: List<ShareUser> = emptyList(),
    val isLoading: Boolean = false,
    val isLoadingMore: Boolean = false,
    val error: String? = null,
    val hasMore: Boolean = true,
    val currentPage: Int = 1,
    val showCreateDialog: Boolean = false
)

class ShareViewModel(
    private val api: HomedriveApi
) : ViewModel() {

    private val repository = ShareRepository(api)
    private val _uiState = MutableStateFlow(ShareUiState())
    val uiState: StateFlow<ShareUiState> = _uiState.asStateFlow()

    init {
        loadShares()
        loadShareUsers()
    }

    fun loadShares() {
        if (_uiState.value.isLoading) return

        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isLoading = true,
                error = null,
                currentPage = 1
            )

            repository.getShares(1)
                .onSuccess { response ->
                    _uiState.value = _uiState.value.copy(
                        shares = response.data,
                        isLoading = false,
                        hasMore = response.hasMore
                    )
                }
                .onFailure { exception ->
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        error = exception.message ?: "加载分享失败"
                    )
                }
        }
    }

    fun loadMoreShares() {
        if (_uiState.value.isLoadingMore || !_uiState.value.hasMore) return

        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isLoadingMore = true,
                currentPage = _uiState.value.currentPage + 1
            )

            val nextPage = _uiState.value.currentPage
            repository.getShares(nextPage)
                .onSuccess { response ->
                    _uiState.value = _uiState.value.copy(
                        shares = _uiState.value.shares + response.data,
                        isLoadingMore = false,
                        hasMore = response.hasMore
                    )
                }
                .onFailure { exception ->
                    _uiState.value = _uiState.value.copy(
                        isLoadingMore = false,
                        error = exception.message
                    )
                }
        }
    }

    fun loadShareUsers() {
        viewModelScope.launch {
            repository.getShareUsers()
                .onSuccess { users ->
                    _uiState.value = _uiState.value.copy(shareUsers = users)
                }
                .onFailure { /* Ignore */ }
        }
    }

    fun createShare(fileId: Long, userIds: List<Long>, expiresAt: String? = null) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(showCreateDialog = false)

            repository.createShare(fileId, userIds, expiresAt)
                .onSuccess {
                    loadShares()
                }
                .onFailure { exception ->
                    _uiState.value = _uiState.value.copy(
                        error = exception.message ?: "创建分享失败"
                    )
                }
        }
    }

    fun deleteShare(shareId: Long) {
        viewModelScope.launch {
            repository.deleteShare(shareId)
                .onSuccess {
                    _uiState.value = _uiState.value.copy(
                        shares = _uiState.value.shares.filter { it.id != shareId }
                    )
                }
                .onFailure { exception ->
                    _uiState.value = _uiState.value.copy(
                        error = exception.message ?: "删除分享失败"
                    )
                }
        }
    }

    fun showCreateDialog() {
        _uiState.value = _uiState.value.copy(showCreateDialog = true)
    }

    fun hideCreateDialog() {
        _uiState.value = _uiState.value.copy(showCreateDialog = false)
    }

    fun clearError() {
        _uiState.value = _uiState.value.copy(error = null)
    }
}
