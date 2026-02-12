package com.kingzcheung.homedrive.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kingzcheung.homedrive.data.api.HomedriveApi
import com.kingzcheung.homedrive.data.local.PreferencesManager
import com.kingzcheung.homedrive.data.model.Album
import com.kingzcheung.homedrive.data.model.AlbumResponse
import com.kingzcheung.homedrive.data.model.CreateAlbumRequest
import com.kingzcheung.homedrive.data.model.FileItem
import com.kingzcheung.homedrive.data.model.FileType
import com.kingzcheung.homedrive.data.repository.AlbumRepository
import com.kingzcheung.homedrive.data.repository.FileRepository
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
    val currentPage: Int = 1,
    val showCreateDialog: Boolean = false,
    val createSuccess: Boolean = false,
    // 删除相册确认对话框
    val showDeleteDialog: Boolean = false,
    val albumToDelete: Album? = null,
    // 图片选择弹窗状态
    val showAddFilesDialog: Boolean = false,
    val selectedFileIds: Set<Long> = emptySet(),
    val isAddingFiles: Boolean = false,
    // 远程图片列表（用于选择添加到相册）
    val availableFiles: List<FileItem> = emptyList(),
    val isLoadingFiles: Boolean = false,
    val filesPage: Int = 1,
    val hasMoreFiles: Boolean = true,
    // 当前正在添加图片的相册 ID（可以是新创建的或已有的）
    val targetAlbumId: Long? = null,
    val targetAlbumName: String = "",
    // 媒体查看器状态
    val showMediaViewer: Boolean = false,
    val selectedFileIndex: Int = 0
)

class AlbumViewModel(
    private val api: HomedriveApi,
    private val preferencesManager: PreferencesManager
) : ViewModel() {

    private val repository = AlbumRepository(api, preferencesManager)
    private val fileRepository = FileRepository(api, preferencesManager)
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

    fun refresh() {
        // 重置状态并重新加载
        _uiState.value = _uiState.value.copy(
            currentPage = 1,
            hasMore = true,
            error = null
        )
        loadAlbums()
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
            albumFiles = emptyList(),
            showMediaViewer = false
        )
    }

    // 媒体查看器相关方法
    fun openMediaViewer(fileIndex: Int) {
        _uiState.value = _uiState.value.copy(
            showMediaViewer = true,
            selectedFileIndex = fileIndex
        )
    }

    fun closeMediaViewer() {
        _uiState.value = _uiState.value.copy(
            showMediaViewer = false,
            selectedFileIndex = 0
        )
    }

    fun clearError() {
        _uiState.value = _uiState.value.copy(error = null)
    }

    fun showCreateDialog() {
        _uiState.value = _uiState.value.copy(showCreateDialog = true)
    }

    fun hideCreateDialog() {
        _uiState.value = _uiState.value.copy(showCreateDialog = false)
    }

    fun showDeleteDialog(album: Album) {
        _uiState.value = _uiState.value.copy(
            showDeleteDialog = true,
            albumToDelete = album
        )
    }

    fun hideDeleteDialog() {
        _uiState.value = _uiState.value.copy(
            showDeleteDialog = false,
            albumToDelete = null
        )
    }

    fun deleteAlbum() {
        val album = _uiState.value.albumToDelete ?: return

        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoading = true)

            repository.deleteAlbum(album.id)
                .onSuccess {
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        showDeleteDialog = false,
                        albumToDelete = null,
                        albums = _uiState.value.albums.filter { it.id != album.id }
                    )
                }
                .onFailure { exception ->
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        error = exception.message ?: "删除相册失败"
                    )
                }
        }
    }

    fun createAlbum(name: String, description: String? = null) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoading = true)
            
            repository.createAlbum(CreateAlbumRequest(name, description))
                .onSuccess { album ->
                    val newAlbum = Album(
                        id = album.id,
                        name = album.name,
                        description = album.description,
                        coverFileId = album.coverFileId,
                        fileCount = album.fileCount,
                        createdAt = album.createdAt,
                        updatedAt = album.updatedAt
                    )
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        showCreateDialog = false,
                        createSuccess = true,
                        albums = _uiState.value.albums + newAlbum,
                        // 创建成功后显示添加图片对话框
                        targetAlbumId = album.id,
                        targetAlbumName = album.name,
                        showAddFilesDialog = true,
                        selectedFileIds = emptySet(),
                        availableFiles = emptyList(),
                        filesPage = 1,
                        hasMoreFiles = true
                    )
                    // 加载远程图片列表
                    loadAvailableFiles()
                }
                .onFailure { exception ->
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        error = exception.message ?: "创建相册失败"
                    )
                }
        }
    }

    fun resetCreateSuccess() {
        _uiState.value = _uiState.value.copy(createSuccess = false)
    }

    // 从相册详情页打开添加图片弹窗
    fun showAddFilesToCurrentAlbum() {
        val album = _uiState.value.currentAlbum ?: return
        _uiState.value = _uiState.value.copy(
            targetAlbumId = album.id,
            targetAlbumName = album.name,
            showAddFilesDialog = true,
            selectedFileIds = emptySet(),
            availableFiles = emptyList(),
            filesPage = 1,
            hasMoreFiles = true
        )
        loadAvailableFiles()
    }

    // 加载远程图片列表
    private fun loadAvailableFiles() {
        if (_uiState.value.isLoadingFiles) return

        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoadingFiles = true)

            fileRepository.getFiles("/", _uiState.value.filesPage)
                .onSuccess { response ->
                    // 只显示图片和视频类型的文件
                    val mediaFiles = response.data.filter {
                        it.type == FileType.IMAGE || it.type == FileType.VIDEO
                    }
                    _uiState.value = _uiState.value.copy(
                        availableFiles = _uiState.value.availableFiles + mediaFiles,
                        isLoadingFiles = false,
                        filesPage = _uiState.value.filesPage + 1,
                        hasMoreFiles = response.hasMore
                    )
                }
                .onFailure { exception ->
                    _uiState.value = _uiState.value.copy(
                        isLoadingFiles = false,
                        error = exception.message ?: "加载图片列表失败"
                    )
                }
        }
    }

    // 加载更多图片
    fun loadMoreAvailableFiles() {
        if (!_uiState.value.hasMoreFiles || _uiState.value.isLoadingFiles) return
        loadAvailableFiles()
    }

    // 图片选择相关方法
    fun toggleFileSelection(fileId: Long) {
        val currentSelection = _uiState.value.selectedFileIds
        val newSelection = if (currentSelection.contains(fileId)) {
            currentSelection - fileId
        } else {
            currentSelection + fileId
        }
        _uiState.value = _uiState.value.copy(selectedFileIds = newSelection)
    }

    fun hideAddFilesDialog() {
        _uiState.value = _uiState.value.copy(
            showAddFilesDialog = false,
            targetAlbumId = null,
            targetAlbumName = "",
            selectedFileIds = emptySet(),
            availableFiles = emptyList(),
            filesPage = 1,
            hasMoreFiles = true
        )
    }

    fun addSelectedFilesToAlbum() {
        val albumId = _uiState.value.targetAlbumId ?: return
        val fileIds = _uiState.value.selectedFileIds.toList()
        if (fileIds.isEmpty()) {
            hideAddFilesDialog()
            return
        }

        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isAddingFiles = true)
            
            repository.addFilesToAlbum(albumId, fileIds)
                .onSuccess {
                    _uiState.value = _uiState.value.copy(
                        isAddingFiles = false,
                        showAddFilesDialog = false,
                        targetAlbumId = null,
                        targetAlbumName = "",
                        selectedFileIds = emptySet(),
                        availableFiles = emptyList()
                    )
                    // 刷新相册列表以更新文件计数
                    loadAlbums()
                    // 如果当前在相册详情页，刷新相册文件列表
                    if (_uiState.value.currentAlbum?.id == albumId) {
                        openAlbum(_uiState.value.currentAlbum!!)
                    }
                }
                .onFailure { exception ->
                    _uiState.value = _uiState.value.copy(
                        isAddingFiles = false,
                        error = exception.message ?: "添加图片失败"
                    )
                }
        }
    }
}
