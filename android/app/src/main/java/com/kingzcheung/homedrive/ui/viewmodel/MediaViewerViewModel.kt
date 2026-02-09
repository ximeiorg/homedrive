package com.kingzcheung.homedrive.ui.viewmodel

import androidx.lifecycle.ViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

data class MediaViewerUiState(
    val isLoading: Boolean = false,
    val error: String? = null,
    val currentIndex: Int = 0,
    val isFullscreen: Boolean = true
)

class MediaViewerViewModel : ViewModel() {

    private val _uiState = MutableStateFlow(MediaViewerUiState())
    val uiState: StateFlow<MediaViewerUiState> = _uiState.asStateFlow()

    fun setCurrentIndex(index: Int) {
        _uiState.value = _uiState.value.copy(currentIndex = index)
    }

    fun toggleFullscreen() {
        _uiState.value = _uiState.value.copy(
            isFullscreen = !_uiState.value.isFullscreen
        )
    }

    fun releasePlayer() {
        // Player cleanup is handled by the composable
    }

    fun clearError() {
        _uiState.value = _uiState.value.copy(error = null)
    }
}
