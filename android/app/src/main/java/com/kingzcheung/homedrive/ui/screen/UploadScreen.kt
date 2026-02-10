package com.kingzcheung.homedrive.ui.screen

import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.InsertDriveFile
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.kingzcheung.homedrive.R
import com.kingzcheung.homedrive.ui.viewmodel.UploadFile
import com.kingzcheung.homedrive.ui.viewmodel.UploadState
import com.kingzcheung.homedrive.ui.viewmodel.UploadViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun UploadScreen(
    onNavigateBack: () -> Unit,
    viewModel: UploadViewModel
) {
    val uiState by viewModel.uiState.collectAsState()
    val context = LocalContext.current

    val imagePickerLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.GetMultipleContents()
    ) { uris ->
        if (uris.isNotEmpty()) {
            viewModel.addFiles(uris, context)
        }
    }

    val videoPickerLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.GetMultipleContents()
    ) { uris ->
        if (uris.isNotEmpty()) {
            viewModel.addFiles(uris, context)
        }
    }

    Scaffold(
        containerColor = MaterialTheme.colorScheme.background,
        topBar = {
            CenterAlignedTopAppBar(
                title = { 
                    Text(
                        stringResource(R.string.upload),
                        style = MaterialTheme.typography.titleMedium
                    ) 
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = stringResource(R.string.back)
                        )
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = Color.Transparent,
                    titleContentColor = MaterialTheme.colorScheme.onBackground,
                    navigationIconContentColor = MaterialTheme.colorScheme.onBackground
                )
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(horizontal = 20.dp, vertical = 8.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            // 选择文件按钮
            UploadSection(title = "选择文件") {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Surface(
                        onClick = { imagePickerLauncher.launch("image/*") },
                        modifier = Modifier.weight(1f),
                        shape = RoundedCornerShape(24.dp),
                        color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.5f)
                    ) {
                        Column(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(16.dp),
                            horizontalAlignment = Alignment.CenterHorizontally
                        ) {
                            Icon(
                                Icons.Default.Image,
                                contentDescription = null,
                                modifier = Modifier.size(28.dp),
                                tint = MaterialTheme.colorScheme.primary
                            )
                            Spacer(modifier = Modifier.height(6.dp))
                            Text(
                                "选择图片",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurface
                            )
                        }
                    }

                    Surface(
                        onClick = { videoPickerLauncher.launch("video/*") },
                        modifier = Modifier.weight(1f),
                        shape = RoundedCornerShape(24.dp),
                        color = MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.5f)
                    ) {
                        Column(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(16.dp),
                            horizontalAlignment = Alignment.CenterHorizontally
                        ) {
                            Icon(
                                Icons.Default.VideoLibrary,
                                contentDescription = null,
                                modifier = Modifier.size(28.dp),
                                tint = MaterialTheme.colorScheme.secondary
                            )
                            Spacer(modifier = Modifier.height(6.dp))
                            Text(
                                "选择视频",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurface
                            )
                        }
                    }
                }
            }

            // 上传进度（只在有文件时显示）
            if (uiState.files.isNotEmpty()) {
                UploadSection(title = "上传进度") {
                    Surface(
                        shape = RoundedCornerShape(20.dp),
                        color = MaterialTheme.colorScheme.surface
                    ) {
                        Column(
                            modifier = Modifier.padding(16.dp)
                        ) {
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween,
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Text(
                                    text = if (uiState.isUploading) "正在上传 ${uiState.currentFileIndex + 1} / ${uiState.files.size}"
                                           else "已选择 ${uiState.files.size} 个文件",
                                    style = MaterialTheme.typography.titleSmall
                                )
                                if (uiState.isUploading) {
                                    Text(
                                        text = "${uiState.uploadProgress}%",
                                        style = MaterialTheme.typography.titleSmall,
                                        color = MaterialTheme.colorScheme.primary
                                    )
                                }
                            }

                            if (uiState.isUploading) {
                                Spacer(modifier = Modifier.height(12.dp))

                                LinearProgressIndicator(
                                    progress = { uiState.uploadProgress / 100f },
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .height(6.dp),
                                    color = MaterialTheme.colorScheme.primary,
                                    trackColor = MaterialTheme.colorScheme.surfaceVariant,
                                )

                                Spacer(modifier = Modifier.height(12.dp))

                                Row(
                                    modifier = Modifier.fillMaxWidth(),
                                    horizontalArrangement = Arrangement.spacedBy(16.dp)
                                ) {
                                    Row(verticalAlignment = Alignment.CenterVertically) {
                                        Icon(
                                            Icons.Default.Check,
                                            contentDescription = null,
                                            modifier = Modifier.size(16.dp),
                                            tint = MaterialTheme.colorScheme.primary
                                        )
                                        Spacer(modifier = Modifier.width(4.dp))
                                        Text(
                                            text = "成功: ${uiState.successCount}",
                                            style = MaterialTheme.typography.bodySmall,
                                            color = MaterialTheme.colorScheme.primary
                                        )
                                    }
                                    Row(verticalAlignment = Alignment.CenterVertically) {
                                        Icon(
                                            Icons.Default.Error,
                                            contentDescription = null,
                                            modifier = Modifier.size(16.dp),
                                            tint = MaterialTheme.colorScheme.error
                                        )
                                        Spacer(modifier = Modifier.width(4.dp))
                                        Text(
                                            text = "失败: ${uiState.errorCount}",
                                            style = MaterialTheme.typography.bodySmall,
                                            color = MaterialTheme.colorScheme.error
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 文件列表（可滚动）
            if (uiState.files.isNotEmpty()) {
                LazyColumn(
                    modifier = Modifier.weight(1f),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    itemsIndexed(uiState.files, key = { index, file -> "$index-${file.name}" }) { index, file ->
                        UploadFileItem(
                            file = file,
                            onRemove = { viewModel.removeFile(index) }
                        )
                    }
                }
            }

            // 底部操作按钮（始终显示在有文件时）
            if (uiState.files.isNotEmpty()) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    // 清空按钮
                    OutlinedButton(
                        onClick = { viewModel.clearAllFiles() },
                        modifier = Modifier.weight(1f),
                        enabled = !uiState.isUploading,
                        shape = RoundedCornerShape(24.dp)
                    ) {
                        Icon(
                            Icons.Default.DeleteSweep,
                            contentDescription = null,
                            modifier = Modifier.size(18.dp)
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text("清空列表")
                    }

                    // 开始上传按钮
                    Button(
                        onClick = { viewModel.startUpload() },
                        modifier = Modifier.weight(1f),
                        enabled = !uiState.isUploading && uiState.files.isNotEmpty(),
                        shape = RoundedCornerShape(24.dp)
                    ) {
                        if (uiState.isUploading) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(18.dp),
                                strokeWidth = 2.dp,
                                color = MaterialTheme.colorScheme.onPrimary
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text("上传中...")
                        } else {
                            Icon(
                                Icons.Default.Upload,
                                contentDescription = null,
                                modifier = Modifier.size(18.dp)
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text("开始上传")
                        }
                    }
                }
            }

            // 空状态
            if (uiState.files.isEmpty() && !uiState.isUploading) {
                Spacer(modifier = Modifier.weight(1f))
                Box(
                    modifier = Modifier.fillMaxWidth(),
                    contentAlignment = Alignment.Center
                ) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Surface(
                            shape = RoundedCornerShape(32.dp),
                            color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.3f),
                            modifier = Modifier.size(100.dp)
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Icon(
                                    Icons.Default.CloudUpload,
                                    contentDescription = null,
                                    modifier = Modifier.size(48.dp),
                                    tint = MaterialTheme.colorScheme.primary.copy(alpha = 0.6f)
                                )
                            }
                        }
                        Spacer(modifier = Modifier.height(20.dp))
                        Text(
                            "选择要上传的文件",
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            "支持图片和视频文件",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f)
                        )
                    }
                }
                Spacer(modifier = Modifier.weight(1f))
            }
        }
    }
}

@Composable
private fun UploadSection(
    title: String,
    content: @Composable ColumnScope.() -> Unit
) {
    Column {
        Text(
            text = title,
            style = MaterialTheme.typography.titleSmall,
            color = MaterialTheme.colorScheme.primary,
            modifier = Modifier.padding(start = 4.dp, bottom = 8.dp)
        )
        content()
    }
}

@Composable
private fun UploadFileItem(
    file: UploadFile,
    onRemove: () -> Unit
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(20.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Surface(
                shape = RoundedCornerShape(12.dp),
                color = when {
                    file.type.startsWith("image") -> MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.5f)
                    file.type.startsWith("video") -> MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.5f)
                    else -> MaterialTheme.colorScheme.surfaceVariant
                },
                modifier = Modifier.size(40.dp)
            ) {
                Box(contentAlignment = Alignment.Center) {
                    Icon(
                        imageVector = when {
                            file.type.startsWith("image") -> Icons.Default.Image
                            file.type.startsWith("video") -> Icons.Default.VideoLibrary
                            else -> Icons.AutoMirrored.Filled.InsertDriveFile
                        },
                        contentDescription = null,
                        modifier = Modifier.size(20.dp),
                        tint = when {
                            file.type.startsWith("image") -> MaterialTheme.colorScheme.primary
                            file.type.startsWith("video") -> MaterialTheme.colorScheme.secondary
                            else -> MaterialTheme.colorScheme.onSurfaceVariant
                        }
                    )
                }
            }

            Spacer(modifier = Modifier.width(12.dp))

            Column(
                modifier = Modifier.weight(1f)
            ) {
                Text(
                    text = file.name,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 1
                )
                Spacer(modifier = Modifier.height(2.dp))
                Text(
                    text = formatFileSize(file.size),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )

                if (file.state == UploadState.UPLOADING) {
                    Spacer(modifier = Modifier.height(6.dp))
                    LinearProgressIndicator(
                        progress = { file.progress / 100f },
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(4.dp),
                        color = MaterialTheme.colorScheme.primary,
                        trackColor = MaterialTheme.colorScheme.surfaceVariant,
                    )
                }
            }

            IconButton(
                onClick = onRemove,
                enabled = file.state != UploadState.UPLOADING,
                modifier = Modifier.size(36.dp)
            ) {
                Icon(
                    Icons.Default.Close,
                    contentDescription = stringResource(R.string.remove),
                    modifier = Modifier.size(18.dp),
                    tint = if (file.state != UploadState.UPLOADING) {
                        MaterialTheme.colorScheme.error
                    } else {
                        MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)
                    }
                )
            }
        }
    }
}

private fun formatFileSize(size: Long): String {
    return when {
        size < 1024 -> "$size B"
        size < 1024 * 1024 -> String.format("%.1f KB", size / 1024.0)
        size < 1024 * 1024 * 1024 -> String.format("%.1f MB", size / (1024.0 * 1024))
        else -> String.format("%.1f GB", size / (1024.0 * 1024 * 1024))
    }
}
