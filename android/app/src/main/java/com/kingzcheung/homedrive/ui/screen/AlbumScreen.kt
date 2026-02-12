package com.kingzcheung.homedrive.ui.screen

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.itemsIndexed
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import coil.request.ImageRequest
import com.kingzcheung.homedrive.R
import com.kingzcheung.homedrive.data.model.Album
import com.kingzcheung.homedrive.data.model.FileItem
import com.kingzcheung.homedrive.data.model.FileType
import com.kingzcheung.homedrive.di.AppContainer
import com.kingzcheung.homedrive.ui.viewmodel.AlbumViewModel
import com.kingzcheung.homedrive.ui.viewmodel.MediaViewerViewModel
import kotlinx.coroutines.launch
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AlbumScreen(
    onNavigateBack: () -> Unit,
    onAlbumClick: (Album) -> Unit,
    viewModel: AlbumViewModel,
    modifier: Modifier = Modifier
) {
    val uiState by viewModel.uiState.collectAsState()
    val gridState = rememberLazyGridState()
    var isRefreshing by remember { mutableStateOf(false) }

    // 处理刷新状态
    LaunchedEffect(uiState.isLoading) {
        if (!uiState.isLoading) {
            isRefreshing = false
        }
    }

    LaunchedEffect(gridState) {
        snapshotFlow { gridState.layoutInfo.visibleItemsInfo.lastOrNull()?.index }
            .collect { lastIndex ->
                if (lastIndex != null && lastIndex >= uiState.albums.size - 5) {
                    viewModel.loadMoreAlbums()
                }
            }
    }

    Box(modifier = modifier.fillMaxSize()) {
        PullToRefreshBox(
            isRefreshing = isRefreshing,
            onRefresh = {
                isRefreshing = true
                viewModel.refresh()
            },
            modifier = Modifier.fillMaxSize()
        ) {
            Column(modifier = Modifier.fillMaxSize()) {
                // 当进入相册详情时显示顶部栏
                if (uiState.currentAlbum != null) {
                    TopAppBar(
                        title = { Text(uiState.currentAlbum!!.name) },
                        navigationIcon = {
                            IconButton(onClick = { viewModel.closeAlbum() }) {
                                Icon(
                                    Icons.AutoMirrored.Filled.ArrowBack,
                                    contentDescription = "返回"
                                )
                            }
                        },
                        colors = TopAppBarDefaults.topAppBarColors(
                            containerColor = MaterialTheme.colorScheme.surface.copy(alpha = 0.95f)
                        )
                    )
                }

                // 根据当前状态显示相册列表或相册内的图片
                if (uiState.currentAlbum != null) {
                    // 显示当前相册的图片列表
                    AlbumFilesGrid(
                        files = uiState.albumFiles,
                        isLoading = uiState.isLoading,
                        isLoadingMore = uiState.isLoadingMore,
                        hasMore = uiState.hasMore,
                        gridState = gridState,
                        onFileClick = { index, _ ->
                            viewModel.openMediaViewer(index)
                        },
                        onLoadMore = { viewModel.loadMoreAlbumFiles() }
                    )
                } else {
                    // 显示相册列表
                    AlbumsGrid(
                        albums = uiState.albums,
                        gridState = gridState,
                        onAlbumClick = { album ->
                            viewModel.openAlbum(album)
                        },
                        onAlbumLongClick = { album ->
                            viewModel.showDeleteDialog(album)
                        }
                    )
                }
            }
        }

        // 加载状态覆盖层
        when {
            uiState.isLoading && uiState.albums.isEmpty() -> {
                CircularProgressIndicator(
                    modifier = Modifier.align(Alignment.Center)
                )
            }
            uiState.error != null && uiState.albums.isEmpty() -> {
                ErrorContent(
                    error = uiState.error!!,
                    onRetry = { viewModel.loadAlbums() },
                    modifier = Modifier.align(Alignment.Center)
                )
            }
            uiState.albums.isEmpty() -> {
                EmptyContent(
                    message = stringResource(R.string.empty_albums),
                    modifier = Modifier.align(Alignment.Center)
                )
            }
        }
    }

    // 全屏媒体查看器
    if (uiState.showMediaViewer && uiState.albumFiles.isNotEmpty()) {
        val mediaViewerViewModel = rememberMediaViewerViewModel()
        AlbumMediaViewer(
            files = uiState.albumFiles,
            initialIndex = uiState.selectedFileIndex,
            onNavigateBack = { viewModel.closeMediaViewer() },
            viewModel = mediaViewerViewModel
        )
    }

    // 创建相册对话框
    if (uiState.showCreateDialog) {
        CreateAlbumDialog(
            onDismiss = { viewModel.hideCreateDialog() },
            onCreate = { name, description ->
                viewModel.createAlbum(name, description)
            },
            isLoading = uiState.isLoading
        )
    }

    // 删除相册确认对话框
    if (uiState.showDeleteDialog) {
        AlertDialog(
            onDismissRequest = { viewModel.hideDeleteDialog() },
            title = { Text("删除相册") },
            text = {
                Text("确定要删除相册 \"${uiState.albumToDelete?.name}\" 吗？此操作不可恢复。")
            },
            confirmButton = {
                Button(
                    onClick = { viewModel.deleteAlbum() },
                    enabled = !uiState.isLoading
                ) {
                    if (uiState.isLoading) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(16.dp),
                            strokeWidth = 2.dp
                        )
                    } else {
                        Text("删除")
                    }
                }
            },
            dismissButton = {
                TextButton(
                    onClick = { viewModel.hideDeleteDialog() },
                    enabled = !uiState.isLoading
                ) {
                    Text("取消")
                }
            }
        )
    }

    // 添加图片到底部弹窗
    if (uiState.showAddFilesDialog) {
        AddFilesToAlbumBottomSheet(
            albumName = uiState.targetAlbumName,
            files = uiState.availableFiles,
            selectedFileIds = uiState.selectedFileIds,
            isLoading = uiState.isLoadingFiles,
            isAdding = uiState.isAddingFiles,
            hasMore = uiState.hasMoreFiles,
            onDismiss = { viewModel.hideAddFilesDialog() },
            onFileClick = { viewModel.toggleFileSelection(it) },
            onConfirm = { viewModel.addSelectedFilesToAlbum() },
            onLoadMore = { viewModel.loadMoreAvailableFiles() }
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun AddFilesToAlbumBottomSheet(
    albumName: String,
    files: List<FileItem>,
    selectedFileIds: Set<Long>,
    isLoading: Boolean,
    isAdding: Boolean,
    hasMore: Boolean,
    onDismiss: () -> Unit,
    onFileClick: (Long) -> Unit,
    onConfirm: () -> Unit,
    onLoadMore: () -> Unit
) {
    val scope = rememberCoroutineScope()
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val gridState = rememberLazyGridState()
    val context = LocalContext.current

    // 检测滚动到底部加载更多
    LaunchedEffect(gridState) {
        snapshotFlow { gridState.layoutInfo.visibleItemsInfo.lastOrNull()?.index }
            .collect { lastIndex ->
                if (lastIndex != null && lastIndex >= files.size - 10 && hasMore && !isLoading) {
                    onLoadMore()
                }
            }
    }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = MaterialTheme.colorScheme.surface
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .fillMaxHeight(0.7f)
        ) {
            // 标题栏
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = "添加图片到 \"$albumName\"",
                    style = MaterialTheme.typography.titleMedium,
                    modifier = Modifier.weight(1f)
                )
                if (selectedFileIds.isNotEmpty()) {
                    AssistChip(
                        onClick = { },
                        label = { Text("已选 ${selectedFileIds.size}") },
                        modifier = Modifier.padding(end = 8.dp)
                    )
                }
            }

            // 图片网格
            Box(modifier = Modifier.weight(1f)) {
                if (files.isEmpty() && isLoading) {
                    CircularProgressIndicator(
                        modifier = Modifier.align(Alignment.Center)
                    )
                } else if (files.isEmpty()) {
                    Column(
                        modifier = Modifier.align(Alignment.Center),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Icon(
                            Icons.Default.PhotoLibrary,
                            contentDescription = null,
                            modifier = Modifier.size(48.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = "暂无图片",
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                } else {
                    LazyVerticalGrid(
                        columns = GridCells.Adaptive(minSize = 100.dp),
                        state = gridState,
                        contentPadding = PaddingValues(4.dp),
                        horizontalArrangement = Arrangement.spacedBy(4.dp),
                        verticalArrangement = Arrangement.spacedBy(4.dp),
                        modifier = Modifier.fillMaxSize()
                    ) {
                        items(files, key = { it.id }) { file ->
                            SelectableFileItem(
                                file = file,
                                isSelected = selectedFileIds.contains(file.id),
                                onClick = { onFileClick(file.id) }
                            )
                        }
                        
                        // 加载更多指示器
                        if (isLoading && hasMore) {
                            item {
                                Box(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(16.dp),
                                    contentAlignment = Alignment.Center
                                ) {
                                    CircularProgressIndicator(modifier = Modifier.size(24.dp))
                                }
                            }
                        }
                    }
                }
            }

            // 底部按钮
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                OutlinedButton(
                    onClick = onDismiss,
                    modifier = Modifier.weight(1f),
                    enabled = !isAdding
                ) {
                    Text("取消")
                }
                Button(
                    onClick = onConfirm,
                    modifier = Modifier.weight(1f),
                    enabled = selectedFileIds.isNotEmpty() && !isAdding
                ) {
                    if (isAdding) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(16.dp),
                            strokeWidth = 2.dp,
                            color = MaterialTheme.colorScheme.onPrimary
                        )
                    } else {
                        Text("添加 (${selectedFileIds.size})")
                    }
                }
            }
        }
    }
}

@Composable
private fun SelectableFileItem(
    file: FileItem,
    isSelected: Boolean,
    onClick: () -> Unit
) {
    val context = LocalContext.current

    Box(
        modifier = Modifier
            .aspectRatio(1f)
            .clip(MaterialTheme.shapes.small)
            .clickable(onClick = onClick)
    ) {
        // 图片/视频缩略图 - 优先使用 thumbnail，其次使用 url（如果是图片），最后显示占位符
        val imageUrl = when {
            file.thumbnail != null -> file.thumbnail
            file.type == FileType.IMAGE && file.url != null -> file.url
            file.type == FileType.VIDEO && file.thumbnail != null -> file.thumbnail
            else -> null
        }
        if (imageUrl != null) {
            AsyncImage(
                model = ImageRequest.Builder(context)
                    .data(rememberAlbumStaticUrl(imageUrl))
                    .crossfade(true)
                    .build(),
                contentDescription = null,
                contentScale = ContentScale.Crop,
                modifier = Modifier.fillMaxSize()
            )
        } else {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(MaterialTheme.colorScheme.surfaceVariant),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = if (file.type == FileType.VIDEO) Icons.Default.VideoFile else Icons.Default.Image,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }

        // 选中状态遮罩
        if (isSelected) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.3f))
            )
            // 选中图标
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(4.dp),
                contentAlignment = Alignment.TopEnd
            ) {
                Icon(
                    Icons.Default.CheckCircle,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.size(24.dp)
                )
            }
        }
    }
}

@Composable
private fun rememberAlbumStaticUrl(url: String?): String {
    val scope = rememberCoroutineScope()
    var staticUrl by remember { mutableStateOf("") }
    
    LaunchedEffect(url) {
        if (url != null) {
            scope.launch {
                staticUrl = AppContainer.getStaticUrl(url)
            }
        }
    }
    
    return staticUrl
}

@Composable
private fun CreateAlbumDialog(
    onDismiss: () -> Unit,
    onCreate: (String, String?) -> Unit,
    isLoading: Boolean
) {
    var name by remember { mutableStateOf("") }
    var description by remember { mutableStateOf("") }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("创建相册") },
        text = {
            Column {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("相册名称") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
                Spacer(modifier = Modifier.height(8.dp))
                OutlinedTextField(
                    value = description,
                    onValueChange = { description = it },
                    label = { Text("描述（可选）") },
                    singleLine = false,
                    maxLines = 3,
                    modifier = Modifier.fillMaxWidth()
                )
            }
        },
        confirmButton = {
            Button(
                onClick = {
                    if (name.isNotBlank()) {
                        onCreate(name, description.ifBlank { null })
                    }
                },
                enabled = name.isNotBlank() && !isLoading
            ) {
                if (isLoading) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(16.dp),
                        strokeWidth = 2.dp
                    )
                } else {
                    Text("创建")
                }
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss, enabled = !isLoading) {
                Text("取消")
            }
        }
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun AlbumsGrid(
    albums: List<Album>,
    gridState: androidx.compose.foundation.lazy.grid.LazyGridState,
    onAlbumClick: (Album) -> Unit,
    onAlbumLongClick: (Album) -> Unit
) {
    // 如果当前有打开的相册，不显示相册列表
    if (albums.isEmpty()) {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center
        ) {
            EmptyContent(message = stringResource(R.string.empty_albums))
        }
        return
    }

    LazyVerticalGrid(
        columns = GridCells.Adaptive(minSize = 150.dp),
        state = gridState,
        contentPadding = PaddingValues(
            start = 16.dp,
            top = 96.dp, // 为状态栏和顶部导航栏留出空间
            end = 16.dp,
            bottom = 100.dp  // 底部导航栏空间
        ),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
        modifier = Modifier.fillMaxSize()
    ) {
        items(albums, key = { it.id }) { album ->
            AlbumCard(
                album = album,
                onClick = { onAlbumClick(album) },
                onLongClick = { onAlbumLongClick(album) }
            )
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun AlbumCard(
    album: Album,
    onClick: () -> Unit,
    onLongClick: () -> Unit
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .aspectRatio(1f)
            .clip(MaterialTheme.shapes.medium)
            .combinedClickable(
                onClick = onClick,
                onLongClick = onLongClick
            ),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
    ) {
        Column {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .weight(1f)
            ) {
                // 封面图片
                if (album.coverUrl != null) {
                    AsyncImage(
                        model = ImageRequest.Builder(LocalContext.current)
                            .data(rememberAlbumStaticUrl(album.coverUrl))
                            .crossfade(true)
                            .build(),
                        contentDescription = album.name,
                        contentScale = ContentScale.Crop,
                        modifier = Modifier.fillMaxSize()
                    )
                } else {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .background(MaterialTheme.colorScheme.surfaceVariant),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            Icons.Default.PhotoLibrary,
                            contentDescription = null,
                            modifier = Modifier.size(48.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }
            Column(
                modifier = Modifier.padding(12.dp)
            ) {
                Text(
                    text = album.name,
                    style = MaterialTheme.typography.titleSmall,
                    maxLines = 1
                )
                Text(
                    text = stringResource(R.string.photo_count, album.fileCount.toInt()),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun AlbumFilesGrid(
    files: List<FileItem>,
    isLoading: Boolean,
    isLoadingMore: Boolean,
    hasMore: Boolean,
    gridState: androidx.compose.foundation.lazy.grid.LazyGridState,
    onFileClick: (Int, FileItem) -> Unit,
    onLoadMore: () -> Unit
) {
    LaunchedEffect(gridState) {
        snapshotFlow { gridState.layoutInfo.visibleItemsInfo.lastOrNull()?.index }
            .collect { lastIndex ->
                if (lastIndex != null && lastIndex >= files.size - 5 && hasMore && !isLoadingMore) {
                    onLoadMore()
                }
            }
    }

    LazyVerticalGrid(
        columns = GridCells.Adaptive(minSize = 100.dp),
        state = gridState,
        contentPadding = PaddingValues(
            start = 4.dp,
            top = 96.dp, // 为状态栏和顶部导航栏留出空间
            end = 4.dp,
            bottom = 100.dp  // 底部导航栏空间
        ),
        horizontalArrangement = Arrangement.spacedBy(4.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp),
        modifier = Modifier.fillMaxSize()
    ) {
        itemsIndexed(files, key = { _, file -> file.id }) { index, file ->
            FileGridItem(
                file = file,
                isSelected = false,
                onClick = { onFileClick(index, file) },
                onLongClick = { }
            )
        }
    }
}

@Composable
private fun ErrorContent(
    error: String,
    onRetry: () -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier.padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Icon(
            Icons.Default.Error,
            contentDescription = null,
            modifier = Modifier.size(48.dp),
            tint = MaterialTheme.colorScheme.error
        )
        Spacer(modifier = Modifier.height(8.dp))
        Text(
            text = error,
            color = MaterialTheme.colorScheme.error
        )
        Spacer(modifier = Modifier.height(16.dp))
        Button(onClick = onRetry) {
            Text(stringResource(R.string.retry))
        }
    }
}

@Composable
private fun EmptyContent(
    message: String,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier.padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Icon(
            Icons.Default.PhotoLibrary,
            contentDescription = null,
            modifier = Modifier.size(48.dp),
            tint = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Spacer(modifier = Modifier.height(8.dp))
        Text(
            text = message,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

// 创建 MediaViewerViewModel
@Composable
fun rememberMediaViewerViewModel(): MediaViewerViewModel {
    return remember { MediaViewerViewModel() }
}

// 媒体查看器
@Composable
private fun AlbumMediaViewer(
    files: List<FileItem>,
    initialIndex: Int,
    onNavigateBack: () -> Unit,
    viewModel: MediaViewerViewModel
) {
    MediaViewerScreen(
        files = files,
        initialIndex = initialIndex,
        onNavigateBack = onNavigateBack,
        viewModel = viewModel
    )
}
