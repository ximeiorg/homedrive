package com.kingzcheung.homedrive.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kingzcheung.homedrive.data.api.HomedriveApi
import com.kingzcheung.homedrive.data.model.Album
import com.kingzcheung.homedrive.data.model.FileItem
import com.kingzcheung.homedrive.data.repository.AlbumRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class AlbumUiState(
    val albums: List<Album> = emptyList(),
    val currentAlbum: Album? = null,
    val albumFiles: List<FileItem> = emptyList(),
    val isLoading: Boolean = false,
    val isLoadingMore: Boolean = false,
    val error: String? = null,
    val hasMore: Boolean = true,
    val currentPage: Int = 1
)

class AlbumViewModel(
    private val api: HomedriveApi
) : ViewModel() {

    private val repository = AlbumRepository(api)
    private val _uiState = MutableStateFlow(AlbumUiState())
    val uiState: StateFlow<AlbumUiState> = _uiState.asStateFlow()

    init {
        loadAlbums()
    }

    fun loadAlbums() {
        if (_uiState.value.isLoading) return

        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isLoading = true,
                error = null,
                currentPage = 1
            )

            repository.getAlbums(1)
                .onSuccess { response ->
                    _uiState.value = _uiState.value.copy(
                        albums = response.data,
                        isLoading = false,
                        hasMore = response.hasMore
                    )
                }
                .onFailure { exception ->
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        error = exception.message ?: "加载图集失败"
                    )
                }
        }
    }

    fun loadMoreAlbums() {
        if (_uiState.value.isLoadingMore || !_uiState.value.hasMore) return

        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isLoadingMore = true,
                currentPage = _uiState.value.currentPage + 1
            )

            val nextPage = _uiState.value.currentPage
            repository.getAlbums(nextPage)
                .onSuccess { response ->
                    _uiState.value = _uiState.value.copy(
                        albums = _uiState.value.albums + response.data,
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

    fun openAlbum(album: Album) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isLoading = true,
                currentAlbum = album,
                currentPage = 1
            )

            repository.getAlbumFiles(album.id, 1)
                .onSuccess { response ->
                    _uiState.value = _uiState.value.copy(
                        albumFiles = response.data,
                        isLoading = false,
                        hasMore = response.hasMore
                    )
                }
                .onFailure { exception ->
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        error = exception.message ?: "加载图集详情失败"
                    )
                }
        }
    }

    fun loadMoreAlbumFiles() {
        val albumId = _uiState.value.currentAlbum?.id ?: return
        if (_uiState.value.isLoadingMore || !_uiState.value.hasMore) return

        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isLoadingMore = true,
                currentPage = _uiState.value.currentPage + 1
            )

            val nextPage = _uiState.value.currentPage
            repository.getAlbumFiles(albumId, nextPage)
                .onSuccess { response ->
                    _uiState.value = _uiState.value.copy(
                        albumFiles = _uiState.value.albumFiles + response.data,
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

    fun closeAlbum() {
        _uiState.value = _uiState.value.copy(
            currentAlbum = null,
            albumFiles = emptyList()
        )
    }

    fun clearError() {
        _uiState.value = _uiState.value.copy(error = null)
    }
}
