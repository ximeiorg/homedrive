package com.kingzcheung.homedrive.ui.viewmodel

import androidx.lifecycle.ViewModel
import com.kingzcheung.homedrive.data.model.FileItem
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

data class GalleryUiState(
    val files: List<FileItem> = emptyList(),
    val currentTab: Int = 0  // 0 = Photo, 1 = Album
)

class GalleryViewModel : ViewModel() {
    private val _uiState = MutableStateFlow(GalleryUiState())
    val uiState: StateFlow<GalleryUiState> = _uiState.asStateFlow()

    fun onTabSelected(tabIndex: Int) {
        _uiState.value = _uiState.value.copy(currentTab = tabIndex)
    }
}
