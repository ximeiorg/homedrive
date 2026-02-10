package com.kingzcheung.homedrive.ui.screen

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.kingzcheung.homedrive.R
import com.kingzcheung.homedrive.data.model.FileItem
import com.kingzcheung.homedrive.ui.viewmodel.*

sealed class Screen(val route: String, val title: Int, val icon: androidx.compose.ui.graphics.vector.ImageVector) {
    object Gallery : Screen("gallery", R.string.gallery, Icons.Default.PhotoLibrary)
    object Albums : Screen("albums", R.string.albums, Icons.Default.Collections)
    object Shares : Screen("shares", R.string.shares, Icons.Default.Share)
    object Settings : Screen("settings", R.string.settings, Icons.Default.Settings)
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(
    onLogout: () -> Unit,
    galleryViewModel: GalleryViewModel,
    uploadViewModel: UploadViewModel,
    albumViewModel: AlbumViewModel,
    shareViewModel: ShareViewModel,
    settingsViewModel: SettingsViewModel,
    mediaViewerViewModel: MediaViewerViewModel
) {
    val navController = rememberNavController()
    val configuration = LocalConfiguration.current
    val isTv = configuration.screenWidthDp >= 840

    val bottomNavItems = listOf(
        Screen.Gallery,
        Screen.Albums,
        Screen.Shares
    )

    var showUploadDialog by remember { mutableStateOf(false) }
    
    // 媒体查看器状态
    var showMediaViewer by remember { mutableStateOf(false) }
    var selectedFileIndex by remember { mutableStateOf(0) }
    
    // 获取当前文件列表
    val galleryUiState by galleryViewModel.uiState.collectAsState()
    val mediaFiles = galleryUiState.files.filter { it.type == com.kingzcheung.homedrive.data.model.FileType.IMAGE || it.type == com.kingzcheung.homedrive.data.model.FileType.VIDEO }
    
    // 滚动状态 - 用于控制顶部栏透明度
    var scrollProgress by remember { mutableFloatStateOf(0f) }
    val titleAlpha by animateFloatAsState(
        targetValue = if (scrollProgress > 0.3f) 0f else 1f - (scrollProgress / 0.3f),
        animationSpec = tween(durationMillis = 150),
        label = "titleAlpha"
    )
    val topBarAlpha by animateFloatAsState(
        targetValue = if (scrollProgress > 0.5f) 0f else 1f - (scrollProgress / 0.5f) * 0.7f,
        animationSpec = tween(durationMillis = 150),
        label = "topBarAlpha"
    )

    Scaffold(
        topBar = {
            if (!isTv) {
                Surface(
                    modifier = Modifier.fillMaxWidth(),
                    color = MaterialTheme.colorScheme.surface.copy(alpha = topBarAlpha)
                ) {
                    TopAppBar(
                        title = { 
                            Text(
                                "HomeDrive", 
                                style = MaterialTheme.typography.titleMedium,
                                modifier = Modifier.alpha(titleAlpha)
                            ) 
                        },
                        actions = {
                            IconButton(onClick = { navController.navigate(Screen.Settings.route) }) {
                                Icon(
                                    Icons.Default.Settings,
                                    contentDescription = "设置",
                                    modifier = Modifier.size(24.dp)
                                )
                            }
                        },
                        colors = TopAppBarDefaults.topAppBarColors(
                            containerColor = Color.Transparent
                        )
                    )
                }
            }
        },
        bottomBar = {
            if (!isTv) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 8.dp)
                ) {
                    Surface(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(64.dp),
                        shape = MaterialTheme.shapes.large,
                        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.95f),
                        tonalElevation = 8.dp,
                        shadowElevation = 8.dp
                    ) {
                        Row(
                            modifier = Modifier.fillMaxSize(),
                            horizontalArrangement = Arrangement.SpaceEvenly,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            val navBackStackEntry by navController.currentBackStackEntryAsState()
                            val currentDestination = navBackStackEntry?.destination

                            bottomNavItems.forEach { screen ->
                                val isSelected = currentDestination?.hierarchy?.any { it.route == screen.route } == true
                                NavigationBarItem(
                                    icon = { 
                                        Icon(
                                            screen.icon, 
                                            contentDescription = null,
                                            tint = if (isSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                    },
                                    label = { 
                                        Text(
                                            getStringResource(screen.title),
                                            color = if (isSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                    },
                                    selected = isSelected,
                                    onClick = {
                                        navController.navigate(screen.route) {
                                            popUpTo(navController.graph.findStartDestination().id) {
                                                saveState = true
                                            }
                                            launchSingleTop = true
                                            restoreState = true
                                        }
                                    },
                                    colors = NavigationBarItemDefaults.colors(
                                        selectedIconColor = MaterialTheme.colorScheme.primary,
                                        selectedTextColor = MaterialTheme.colorScheme.primary,
                                        unselectedIconColor = MaterialTheme.colorScheme.onSurfaceVariant,
                                        unselectedTextColor = MaterialTheme.colorScheme.onSurfaceVariant,
                                        indicatorColor = Color.Transparent
                                    )
                                )
                            }
                        }
                    }
                }
            }
        },
        floatingActionButton = {
            if (!isTv) {
                FloatingActionButton(
                    onClick = { showUploadDialog = true },
                    containerColor = MaterialTheme.colorScheme.primary
                ) {
                    Icon(Icons.Default.Upload, contentDescription = "上传")
                }
            }
        }
    ) { paddingValues ->
        Row(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues),
            verticalAlignment = Alignment.Top
        ) {
            if (isTv) {
                NavigationRail {
                    Spacer(modifier = Modifier.weight(1f))
                    val navBackStackEntry by navController.currentBackStackEntryAsState()
                    val currentDestination = navBackStackEntry?.destination

                    bottomNavItems.forEach { screen ->
                        NavigationRailItem(
                            icon = { Icon(screen.icon, contentDescription = null) },
                            label = { Text(getStringResource(screen.title)) },
                            selected = currentDestination?.hierarchy?.any { it.route == screen.route } == true,
                            onClick = {
                                navController.navigate(screen.route) {
                                    popUpTo(navController.graph.findStartDestination().id) {
                                        saveState = true
                                    }
                                    launchSingleTop = true
                                    restoreState = true
                                }
                            }
                        )
                    }
                    Spacer(modifier = Modifier.weight(1f))
                }
            }

            NavHost(
                navController = navController,
                startDestination = Screen.Gallery.route,
                modifier = Modifier.fillMaxSize()
            ) {
                composable(Screen.Gallery.route) {
                    GalleryScreen(
                        onNavigateToFolder = { folder ->
                            // Handle folder navigation
                        },
                        onFileClick = { file ->
                            // 找到点击文件在媒体列表中的索引
                            val index = mediaFiles.indexOfFirst { it.id == file.id }
                            if (index >= 0) {
                                selectedFileIndex = index
                                showMediaViewer = true
                            }
                        },
                        onNavigateToUpload = { /* Navigate to upload */ },
                        viewModel = galleryViewModel,
                        onScrollProgressChange = { progress -> scrollProgress = progress }
                    )
                }

                composable(Screen.Albums.route) {
                    AlbumScreen(
                        onNavigateBack = { navController.popBackStack() },
                        onAlbumClick = { /* Navigate to album detail */ },
                        viewModel = albumViewModel
                    )
                }

                composable(Screen.Shares.route) {
                    ShareScreen(
                        onNavigateBack = { navController.popBackStack() },
                        viewModel = shareViewModel
                    )
                }

                composable(Screen.Settings.route) {
                    SettingsScreen(
                        onNavigateBack = { navController.popBackStack() },
                        onLogout = onLogout,
                        viewModel = settingsViewModel
                    )
                }
            }
        }
    }

    // Upload Bottom Sheet
    if (showUploadDialog) {
        UploadBottomSheet(
            onDismiss = { showUploadDialog = false },
            viewModel = uploadViewModel
        )
    }
    
    // 全屏媒体查看器
    if (showMediaViewer && mediaFiles.isNotEmpty()) {
        MediaViewerScreen(
            files = mediaFiles,
            initialIndex = selectedFileIndex,
            onNavigateBack = { showMediaViewer = false },
            viewModel = mediaViewerViewModel
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun UploadBottomSheet(
    onDismiss: () -> Unit,
    viewModel: UploadViewModel
) {
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        containerColor = MaterialTheme.colorScheme.surface
    ) {
        UploadScreen(
            onNavigateBack = onDismiss,
            viewModel = viewModel
        )
    }
}

@Composable
fun UploadDialog(
    onDismiss: () -> Unit,
    viewModel: UploadViewModel
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("上传文件") },
        text = {
            UploadScreen(
                onNavigateBack = onDismiss,
                viewModel = viewModel
            )
        },
        confirmButton = {},
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("关闭")
            }
        }
    )
}

@Composable
private fun getStringResource(id: Int): String {
    return when (id) {
        R.string.gallery -> "图库"
        R.string.albums -> "图集"
        R.string.shares -> "分享"
        R.string.settings -> "设置"
        R.string.upload -> "上传"
        else -> ""
    }
}
