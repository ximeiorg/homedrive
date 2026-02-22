package com.kingzcheung.homedrive.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kingzcheung.homedrive.data.model.FileItem
import com.kingzcheung.homedrive.data.repository.FileRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class GalleryUiState(
    val files: List<FileItem> = emptyList(),
    val currentTab: Int = 0,  // 0 = Photo, 1 = Album
    val isLoading: Boolean = false,
    val isLoadingMore: Boolean = false,
    val error: String? = null,
    val currentPath: String = "/",
    val currentPage: Int = 1,
    val hasMore: Boolean = true
)

class GalleryViewModel(
    private val fileRepository: FileRepository
) : ViewModel() {
    private val _uiState = MutableStateFlow(GalleryUiState())
    val uiState: StateFlow<GalleryUiState> = _uiState.asStateFlow()

    init {
        loadFiles()
    }

    fun loadFiles(path: String = "/", page: Int = 1) {
        viewModelScope.launch {
            if (page == 1) {
                _uiState.value = _uiState.value.copy(isLoading = true, error = null)
            } else {
                _uiState.value = _uiState.value.copy(isLoadingMore = true)
            }
            
            fileRepository.getFiles(path, page).fold(
                onSuccess = { paginatedResponse ->
                    val newFiles = if (page == 1) {
                        paginatedResponse.data
                    } else {
                        _uiState.value.files + paginatedResponse.data
                    }
                    
                    _uiState.value = _uiState.value.copy(
                        files = newFiles,
                        isLoading = false,
                        isLoadingMore = false,
                        currentPath = path,
                        currentPage = page,
                        hasMore = paginatedResponse.hasMore
                    )
                },
                onFailure = { exception ->
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        isLoadingMore = false,
                        error = exception.message ?: "Failed to load files"
                    )
                }
            )
        }
    }

    fun loadMore() {
        val currentState = _uiState.value
        if (!currentState.isLoading && !currentState.isLoadingMore && currentState.hasMore) {
            loadFiles(currentState.currentPath, currentState.currentPage + 1)
        }
    }

    fun navigateToFolder(folder: FileItem) {
        loadFiles(folder.path, page = 1)
    }

    fun navigateBack(): Boolean {
        val currentPath = _uiState.value.currentPath
        if (currentPath == "/") return false
        
        val parentPath = currentPath.substringBeforeLast("/", "")
        loadFiles(if (parentPath.isEmpty()) "/" else parentPath, page = 1)
        return true
    }

    fun onTabSelected(tabIndex: Int) {
        _uiState.value = _uiState.value.copy(currentTab = tabIndex)
    }

    fun refresh() {
        loadFiles(_uiState.value.currentPath, page = 1)
    }

    fun clearError() {
        _uiState.value = _uiState.value.copy(error = null)
    }
}
