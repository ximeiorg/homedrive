package com.kingzcheung.homedrive.ui.screen

import android.view.ViewGroup
import android.widget.FrameLayout
import androidx.compose.animation.*
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.InsertDriveFile
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import coil.compose.SubcomposeAsyncImage
import coil.request.ImageRequest
import com.kingzcheung.homedrive.R
import com.kingzcheung.homedrive.data.model.FileItem
import com.kingzcheung.homedrive.data.model.FileType
import com.kingzcheung.homedrive.ui.viewmodel.MediaViewerViewModel
import androidx.media3.common.MediaItem
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.PlayerView
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MediaViewerScreen(
    files: List<FileItem>,
    initialIndex: Int,
    onNavigateBack: () -> Unit,
    viewModel: MediaViewerViewModel
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    
    // Pager 状态
    val pagerState = rememberPagerState(initialPage = initialIndex, pageCount = { files.size })
    
    // 当前显示的文件
    val currentFile by remember { derivedStateOf { files.getOrNull(pagerState.currentPage) } }
    
    // 控制栏显示状态
    var showControls by remember { mutableStateOf(true) }
    
    // 底部信息面板状态
    var showInfoSheet by remember { mutableStateOf(false) }
    
    // 释放视频播放器
    DisposableEffect(Unit) {
        onDispose {
            viewModel.releasePlayer()
        }
    }

    // 全屏查看器
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
    ) {
        // 图片/视频 Pager
        HorizontalPager(
            state = pagerState,
            modifier = Modifier.fillMaxSize(),
            beyondViewportPageCount = 1  // 预加载相邻页面
        ) { page ->
            val file = files.getOrNull(page) ?: return@HorizontalPager
            
            when (file.type) {
                FileType.IMAGE -> {
                    ZoomableImageViewer(
                        imageUrl = rememberStaticUrl(file.url),
                        onTap = { showControls = !showControls },
                        onSwipeUp = { showInfoSheet = true },
                        modifier = Modifier.fillMaxSize()
                    )
                }
                FileType.VIDEO -> {
                    VideoViewer(
                        videoUrl = rememberStaticUrl(file.url),
                        modifier = Modifier.fillMaxSize()
                    )
                }
                else -> {
                    Box(
                        modifier = Modifier.fillMaxSize(),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            Icons.AutoMirrored.Filled.InsertDriveFile,
                            contentDescription = null,
                            modifier = Modifier.size(64.dp),
                            tint = Color.White
                        )
                    }
                }
            }
        }

        // 顶部控制栏
        AnimatedVisibility(
            visible = showControls,
            enter = fadeIn() + slideInVertically { -it },
            exit = fadeOut() + slideOutVertically { -it },
            modifier = Modifier.align(Alignment.TopCenter)
        ) {
            TopAppBar(
                title = { 
                    Text(
                        text = "${pagerState.currentPage + 1} / ${files.size}",
                        color = Color.White
                    ) 
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(
                            Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = "返回",
                            tint = Color.White
                        )
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = Color.Black.copy(alpha = 0.5f),
                    titleContentColor = Color.White
                )
            )
        }

        // 底部控制栏
        AnimatedVisibility(
            visible = showControls,
            enter = fadeIn() + slideInVertically { it },
            exit = fadeOut() + slideOutVertically { it },
            modifier = Modifier.align(Alignment.BottomCenter)
        ) {
            BottomAppBar(
                containerColor = Color.Black.copy(alpha = 0.5f),
                contentColor = Color.White
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceEvenly
                ) {
                    IconButton(onClick = { /* Share */ }) {
                        Icon(Icons.Default.Share, contentDescription = "分享", tint = Color.White)
                    }
                    IconButton(onClick = { /* Download */ }) {
                        Icon(Icons.Default.Download, contentDescription = "下载", tint = Color.White)
                    }
                    IconButton(onClick = { showInfoSheet = true }) {
                        Icon(Icons.Default.Info, contentDescription = "详情", tint = Color.White)
                    }
                }
            }
        }
    }

    // 底部信息面板
    if (showInfoSheet && currentFile != null) {
        MediaInfoBottomSheet(
            file = currentFile!!,
            onDismiss = { showInfoSheet = false }
        )
    }
}

@Composable
private fun ZoomableImageViewer(
    imageUrl: String,
    onTap: () -> Unit,
    onSwipeUp: () -> Unit,
    modifier: Modifier = Modifier
) {
    // 缩放和平移状态
    var scale by remember { mutableFloatStateOf(1f) }
    var offsetX by remember { mutableFloatStateOf(0f) }
    var offsetY by remember { mutableFloatStateOf(0f) }
    
    // 追踪垂直滑动以检测向上滑动手势
    var totalVerticalDrag by remember { mutableFloatStateOf(0f) }
    
    Box(
        modifier = modifier
            .fillMaxSize()
            .pointerInput(Unit) {
                // 只在缩放时处理变换手势，否则让 Pager 处理水平滑动
                detectTransformGestures { _, pan, zoom, _ ->
                    val newScale = (scale * zoom).coerceIn(1f, 5f)
                    
                    if (newScale > 1f) {
                        // 缩放状态下，允许平移
                        scale = newScale
                        val maxX = (size.width * (scale - 1) / 2)
                        val maxY = (size.height * (scale - 1) / 2)
                        offsetX = (offsetX + pan.x).coerceIn(-maxX, maxX)
                        offsetY = (offsetY + pan.y).coerceIn(-maxY, maxY)
                    } else {
                        // 未缩放时，重置偏移
                        scale = 1f
                        offsetX = 0f
                        offsetY = 0f
                    }
                }
            }
            .pointerInput(Unit) {
                // 检测垂直滑动（向上滑动显示信息）
                detectVerticalDragGestures(
                    onDragStart = { totalVerticalDrag = 0f },
                    onDragEnd = {
                        if (totalVerticalDrag < -200 && scale == 1f) {
                            onSwipeUp()
                        }
                        totalVerticalDrag = 0f
                    },
                    onVerticalDrag = { _, dragAmount ->
                        totalVerticalDrag += dragAmount
                    }
                )
            }
            .clickable(onClick = onTap),
        contentAlignment = Alignment.Center
    ) {
        SubcomposeAsyncImage(
            model = ImageRequest.Builder(LocalContext.current)
                .data(imageUrl)
                .crossfade(true)
                .build(),
            contentDescription = null,
            contentScale = ContentScale.Fit,
            modifier = Modifier
                .fillMaxSize()
                .graphicsLayer {
                    scaleX = scale
                    scaleY = scale
                    translationX = offsetX
                    translationY = offsetY
                }
        )
    }
}

@Composable
private fun VideoViewer(
    videoUrl: String,
    modifier: Modifier = Modifier
) {
    val context = LocalContext.current
    val exoPlayer = remember {
        ExoPlayer.Builder(context).build().apply {
            setMediaItem(MediaItem.fromUri(videoUrl))
            prepare()
            playWhenReady = true
        }
    }

    DisposableEffect(exoPlayer) {
        onDispose {
            exoPlayer.release()
        }
    }

    Box(
        modifier = modifier.fillMaxSize()
    ) {
        AndroidView(
            factory = { ctx ->
                PlayerView(ctx).apply {
                    player = exoPlayer
                    layoutParams = FrameLayout.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        ViewGroup.LayoutParams.MATCH_PARENT
                    )
                }
            },
            modifier = Modifier.fillMaxSize()
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun MediaInfoBottomSheet(
    file: FileItem,
    onDismiss: () -> Unit
) {
    val dateFormat = remember { SimpleDateFormat("yyyy年MM月dd日 HH:mm", Locale.getDefault()) }
    
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        containerColor = MaterialTheme.colorScheme.surface
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            // 文件名
            Text(
                text = file.name,
                style = MaterialTheme.typography.titleLarge,
                modifier = Modifier.padding(bottom = 16.dp)
            )
            
            // 文件信息
            FileInfoRow(label = "类型", value = file.mimeType ?: "未知")
            FileInfoRow(label = "大小", value = formatFileSize(file.size))
            FileInfoRow(label = "创建时间", value = formatDateString(file.createdAt, dateFormat))
            FileInfoRow(label = "修改时间", value = formatDateString(file.updatedAt, dateFormat))
            
            if (file.description?.isNotEmpty() == true) {
                FileInfoRow(label = "描述", value = file.description)
            }
            
            Spacer(modifier = Modifier.height(32.dp))
        }
    }
}

@Composable
private fun FileInfoRow(
    label: String,
    value: String
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 8.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurface
        )
    }
}

private fun formatFileSize(bytes: Long?): String {
    if (bytes == null) return "未知"
    return when {
        bytes < 1024 -> "$bytes B"
        bytes < 1024 * 1024 -> String.format("%.1f KB", bytes / 1024.0)
        bytes < 1024 * 1024 * 1024 -> String.format("%.1f MB", bytes / (1024.0 * 1024))
        else -> String.format("%.1f GB", bytes / (1024.0 * 1024 * 1024))
    }
}

private fun formatDateString(dateString: String?, dateFormat: SimpleDateFormat): String {
    if (dateString == null) return "未知"
    return try {
        // ISO 8601 格式解析
        val instant = java.time.Instant.parse(dateString)
        dateFormat.format(Date.from(instant))
    } catch (e: Exception) {
        dateString
    }
}
