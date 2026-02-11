package com.kingzcheung.homedrive.ui.screen

import android.util.Log
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.InsertDriveFile
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material.icons.filled.Image
import androidx.compose.material.icons.filled.PhotoLibrary
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.VideoLibrary
import androidx.compose.material3.*
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import coil.compose.SubcomposeAsyncImage
import coil.request.ImageRequest
import com.kingzcheung.homedrive.R
import com.kingzcheung.homedrive.data.model.FileItem
import com.kingzcheung.homedrive.data.model.FileType
import com.kingzcheung.homedrive.di.AppContainer
import com.kingzcheung.homedrive.ui.viewmodel.GalleryViewModel
import kotlinx.coroutines.runBlocking

private const val TAG = "GalleryScreen"

/**
 * 获取带认证 token 的静态资源 URL
 */
@Composable
fun rememberStaticUrl(urlOrPath: String?): String {
    if (urlOrPath.isNullOrEmpty()) return ""
    
    return remember(urlOrPath) {
        runBlocking {
            val result = AppContainer.getStaticUrl(urlOrPath)
            Log.d(TAG, "rememberStaticUrl: $urlOrPath -> $result")
            result
        }
    }
}

/**
 * 监听滚动到底部和滚动进度
 */
@Composable
private fun ScrollHandler(
    gridState: LazyGridState,
    onLoadMore: () -> Unit,
    onScrollProgressChange: (Float) -> Unit
) {
    // 计算滚动进度 (0f 到 1f)
    val scrollProgress by remember {
        derivedStateOf {
            val firstVisibleItem = gridState.firstVisibleItemIndex
            val scrollOffset = gridState.firstVisibleItemScrollOffset
            
            // 基于第一个可见项目的位置和偏移量计算进度
            // 滚动超过 2 个项目的高度后达到最大进度
            val itemProgress = firstVisibleItem + (scrollOffset / 300f) // 假设每个项目高度约 300dp
            (itemProgress / 5f).coerceIn(0f, 1f) // 滚动 5 个项目后达到最大进度
        }
    }
    
    // 检测是否需要加载更多
    val shouldLoadMore by remember {
        derivedStateOf {
            val lastVisibleItem = gridState.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0
            val totalItems = gridState.layoutInfo.totalItemsCount
            // 当滚动到倒数第 8 个项目时开始加载更多
            lastVisibleItem >= totalItems - 8 && totalItems > 0
        }
    }
    
    // 更新滚动进度
    LaunchedEffect(scrollProgress) {
        onScrollProgressChange(scrollProgress)
    }
    
    // 触发加载更多
    LaunchedEffect(shouldLoadMore) {
        if (shouldLoadMore) {
            onLoadMore()
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun GalleryScreen(
    viewModel: GalleryViewModel,
    onNavigateToFolder: (com.kingzcheung.homedrive.data.model.FileItem) -> Unit,
    onFileClick: (com.kingzcheung.homedrive.data.model.FileItem) -> Unit,
    onNavigateToUpload: () -> Unit,
    modifier: Modifier = Modifier,
    onScrollProgressChange: (Float) -> Unit = {}
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

    Box(modifier = modifier.fillMaxSize()) {
        PullToRefreshBox(
            isRefreshing = isRefreshing,
            onRefresh = {
                isRefreshing = true
                viewModel.refresh()
            },
            modifier = Modifier.fillMaxSize()
        ) {
            Column(
                modifier = Modifier.fillMaxSize()
            ) {
                // 加载状态
                when {
                    uiState.isLoading && uiState.files.isEmpty() -> {
                        Box(
                            modifier = Modifier
                                .fillMaxSize()
                                .weight(1f),
                            contentAlignment = Alignment.Center
                        ) {
                            CircularProgressIndicator()
                        }
                    }
                    uiState.error != null && uiState.files.isEmpty() -> {
                        Box(
                            modifier = Modifier
                                .fillMaxSize()
                                .weight(1f),
                            contentAlignment = Alignment.Center
                        ) {
                            Column(
                                horizontalAlignment = Alignment.CenterHorizontally
                            ) {
                                Text(
                                    text = uiState.error!!,
                                    color = MaterialTheme.colorScheme.error,
                                    textAlign = TextAlign.Center
                                )
                                Spacer(modifier = Modifier.height(16.dp))
                                Button(onClick = { viewModel.refresh() }) {
                                    Text("重试")
                                }
                            }
                        }
                    }
                    uiState.files.isEmpty() -> {
                        Box(
                            modifier = Modifier
                                .fillMaxSize()
                                .weight(1f),
                            contentAlignment = Alignment.Center
                        ) {
                            Column(
                                horizontalAlignment = Alignment.CenterHorizontally
                            ) {
                                Icon(
                                    imageVector = Icons.Default.PhotoLibrary,
                                    contentDescription = null,
                                    modifier = Modifier.size(64.dp),
                                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                                Spacer(modifier = Modifier.height(16.dp))
                                Text(
                                    text = "暂无文件",
                                    style = MaterialTheme.typography.bodyLarge,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }
                    }
                    else -> {
                        // 照片墙网格布局 - 固定4列
                        Box(modifier = Modifier.weight(1f)) {
                            LazyVerticalGrid(
                                columns = GridCells.Fixed(4),
                                contentPadding = PaddingValues(
                                    start = 2.dp,
                                    end = 2.dp,
                                    top = 96.dp, // 为状态栏和顶部导航栏留出空间
                                    bottom = 80.dp // 为底部导航栏留出空间
                                ),
                                state = gridState,
                                modifier = Modifier.fillMaxSize()
                            ) {
                                items(uiState.files, key = { it.id }) { file ->
                                    FileGridItem(
                                        file = file,
                                        isSelected = false,
                                        onClick = {
                                            when (file.type) {
                                                FileType.FOLDER -> {
                                                    viewModel.navigateToFolder(file)
                                                }
                                                else -> onFileClick(file)
                                            }
                                        },
                                        onLongClick = { /* Handle selection */ }
                                    )
                                }
                                
                                // 加载更多指示器
                                if (uiState.isLoadingMore) {
                                    item {
                                        Box(
                                            modifier = Modifier
                                                .fillMaxWidth()
                                                .padding(16.dp),
                                            contentAlignment = Alignment.Center
                                        ) {
                                            CircularProgressIndicator(
                                                modifier = Modifier.size(24.dp),
                                                strokeWidth = 2.dp
                                            )
                                        }
                                    }
                                }
                            }
                            
                            // 监听滚动状态
                            ScrollHandler(
                                gridState = gridState,
                                onLoadMore = { viewModel.loadMore() },
                                onScrollProgressChange = onScrollProgressChange
                            )
                        }
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FileGridItem(
    file: FileItem,
    isSelected: Boolean,
    onClick: () -> Unit,
    onLongClick: () -> Unit
) {
    Box(
        modifier = Modifier
            .aspectRatio(1f)
            .clickable(onClick = onClick)
            .background(
                if (isSelected) MaterialTheme.colorScheme.primaryContainer
                else MaterialTheme.colorScheme.surface
            )
            .padding(1.dp)
    ) {
        Box(modifier = Modifier.fillMaxSize()) {
            when (file.type) {
                FileType.FOLDER -> {
                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(8.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.Center
                    ) {
                        Icon(
                            Icons.Default.Folder,
                            contentDescription = null,
                            modifier = Modifier.size(32.dp),
                            tint = MaterialTheme.colorScheme.primary
                        )
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            text = file.name,
                            style = MaterialTheme.typography.bodySmall,
                            maxLines = 2,
                            textAlign = TextAlign.Center
                        )
                    }
                }
                FileType.IMAGE -> {
                    val imageUrl = rememberStaticUrl(file.url)
                    Log.d(TAG, "Loading image for ${file.name}: $imageUrl")
                    
                    SubcomposeAsyncImage(
                        model = ImageRequest.Builder(LocalContext.current)
                            .data(imageUrl)
                            .crossfade(true)
                            .build(),
                        contentDescription = file.name,
                        contentScale = ContentScale.Crop,
                        modifier = Modifier.fillMaxSize(),
                        loading = {
                            Box(
                                modifier = Modifier
                                    .fillMaxSize()
                                    .background(MaterialTheme.colorScheme.surfaceVariant),
                                contentAlignment = Alignment.Center
                            ) {
                                CircularProgressIndicator(
                                    modifier = Modifier.size(24.dp),
                                    strokeWidth = 2.dp
                                )
                            }
                        },
                        error = {
                            Box(
                                modifier = Modifier
                                    .fillMaxSize()
                                    .background(MaterialTheme.colorScheme.surfaceVariant),
                                contentAlignment = Alignment.Center
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Image,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }
                    )
                    
                    if (isSelected) {
                        Icon(
                            Icons.Default.CheckCircle,
                            contentDescription = null,
                            modifier = Modifier
                                .align(Alignment.TopEnd)
                                .padding(4.dp),
                            tint = MaterialTheme.colorScheme.primary
                        )
                    }
                }
                FileType.VIDEO -> {
                    val thumbnailUrl = rememberStaticUrl(file.thumbnail ?: file.url)
                    Log.d(TAG, "Loading video thumbnail for ${file.name}: $thumbnailUrl")
                    
                    SubcomposeAsyncImage(
                        model = ImageRequest.Builder(LocalContext.current)
                            .data(thumbnailUrl)
                            .crossfade(true)
                            .build(),
                        contentDescription = file.name,
                        contentScale = ContentScale.Crop,
                        modifier = Modifier.fillMaxSize(),
                        loading = {
                            Box(
                                modifier = Modifier
                                    .fillMaxSize()
                                    .background(MaterialTheme.colorScheme.surfaceVariant),
                                contentAlignment = Alignment.Center
                            ) {
                                CircularProgressIndicator(
                                    modifier = Modifier.size(24.dp),
                                    strokeWidth = 2.dp
                                )
                            }
                        },
                        error = {
                            Box(
                                modifier = Modifier
                                    .fillMaxSize()
                                    .background(MaterialTheme.colorScheme.surfaceVariant),
                                contentAlignment = Alignment.Center
                            ) {
                                Icon(
                                    imageVector = Icons.Default.VideoLibrary,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }
                    )
                    
                    // 视频播放图标
                    Icon(
                        Icons.Default.PlayArrow,
                        contentDescription = null,
                        modifier = Modifier
                            .align(Alignment.Center)
                            .size(32.dp)
                            .background(
                                color = Color.Black.copy(alpha = 0.5f),
                                shape = androidx.compose.foundation.shape.CircleShape
                            )
                            .padding(4.dp),
                        tint = Color.White
                    )
                    
                    if (isSelected) {
                        Icon(
                            Icons.Default.CheckCircle,
                            contentDescription = null,
                            modifier = Modifier
                                .align(Alignment.TopEnd)
                                .padding(4.dp),
                            tint = MaterialTheme.colorScheme.primary
                        )
                    }
                }
                else -> {
                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(8.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.Center
                    ) {
                        Icon(
                            Icons.AutoMirrored.Filled.InsertDriveFile,
                            contentDescription = null,
                            modifier = Modifier.size(32.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            text = file.name,
                            style = MaterialTheme.typography.bodySmall,
                            maxLines = 2,
                            textAlign = TextAlign.Center
                        )
                    }
                }
            }
        }
    }
}
