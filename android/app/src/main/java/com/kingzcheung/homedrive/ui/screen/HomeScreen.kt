package com.kingzcheung.homedrive.ui.screen

import android.app.Activity
import android.os.Build
import androidx.activity.compose.BackHandler
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.core.view.WindowCompat
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.kingzcheung.homedrive.R
import com.kingzcheung.homedrive.data.model.FileType
import com.kingzcheung.homedrive.ui.viewmodel.*

sealed class Screen(val route: String, val title: Int, val icon: androidx.compose.ui.graphics.vector.ImageVector) {
    object Gallery : Screen("gallery", R.string.gallery, Icons.Default.PhotoLibrary)
    object Albums : Screen("albums", R.string.albums, Icons.Default.Collections)
    object Shares : Screen("shares", R.string.shares, Icons.Default.Share)
    object Settings : Screen("settings", R.string.settings, Icons.Default.Settings)
    object Upload : Screen("upload", R.string.upload, Icons.Default.Add)
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
    val context = LocalContext.current
    val view = LocalView.current

    // 启用边缘到边缘显示
    val darkTheme = androidx.compose.foundation.isSystemInDarkTheme()
    DisposableEffect(darkTheme) {
        val window = (context as? android.app.Activity)?.window
        window?.let {
            WindowCompat.setDecorFitsSystemWindows(it, false)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                // 在暗黑模式下，状态栏内容使用浅色（白色）
                // 在亮色模式下，状态栏内容使用深色（黑色）
                val appearance = if (darkTheme) {
                    0 // 清除 APPEARANCE_LIGHT_STATUS_BARS，使用浅色内容
                } else {
                    android.view.WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS
                }
                it.insetsController?.setSystemBarsAppearance(
                    appearance,
                    android.view.WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS
                )
            }
        }
        onDispose { }
    }

    val bottomNavItems = listOf(
        Screen.Gallery,
        Screen.Albums,
        Screen.Shares
    )

    var showUploadDialog by remember { mutableStateOf(false) }
    
    // 媒体查看器状态
    var showMediaViewer by remember { mutableStateOf(false) }
    var selectedFileIndex by remember { mutableStateOf(0) }
    
    // 控制底部导航栏显示状态
    var showBottomNav by remember { mutableStateOf(true) }
    
    // 获取当前文件列表
    val galleryUiState by galleryViewModel.uiState.collectAsState()
    val mediaFiles = galleryUiState.files.filter { it.type == FileType.IMAGE || it.type == FileType.VIDEO }
    
    // 滚动状态 - 用于控制顶部栏透明度
    var scrollProgress by remember { mutableFloatStateOf(0f) }
    val titleAlpha by animateFloatAsState(
        targetValue = if (scrollProgress > 0.3f) 0f else 1f - (scrollProgress / 0.3f),
        animationSpec = tween(durationMillis = 150),
        label = "titleAlpha"
    )
    val topBarAlpha by animateFloatAsState(
        targetValue = if (scrollProgress > 0.5f) 0.3f else 1f,
        animationSpec = tween(durationMillis = 150),
        label = "topBarAlpha"
    )

    // 状态栏透明度
    val statusBarAlpha by animateFloatAsState(
        targetValue = if (scrollProgress > 0.3f) 0f else 1f,
        animationSpec = tween(durationMillis = 150),
        label = "statusBarAlpha"
    )
    
    // 获取当前导航栈状态
    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentRoute = navBackStackEntry?.destination?.route
    
    // 处理返回键 - 如果不在起始页面，则返回上一页
    BackHandler(enabled = currentRoute != Screen.Gallery.route) {
        navController.popBackStack()
    }

    Box(modifier = Modifier.fillMaxSize()) {
        Scaffold(
            containerColor = Color.Transparent,
            floatingActionButtonPosition = FabPosition.Center,
            floatingActionButton = {
                if (!isTv) {
                    // 悬浮式底部导航栏 - 只在图库和图集页面显示
                    val navBackStackEntry by navController.currentBackStackEntryAsState()
                    val currentDestination = navBackStackEntry?.destination
                    
                    // 只在图库和图集页面显示导航栏
                    val showNavBar = currentDestination?.hierarchy?.any { 
                        it.route == Screen.Gallery.route || it.route == Screen.Albums.route 
                    } == true
                    
                    if (showNavBar && showBottomNav) {
                        FloatingBottomNavBar(
                            items = listOf(Screen.Gallery, Screen.Albums),
                            currentDestination = currentDestination,
                            onNavigate = { screen ->
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
                }
            }
        ) { paddingValues ->
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(paddingValues)
            ) {
                Row(modifier = Modifier.fillMaxSize()) {
                    // TV 模式下显示侧边导航栏
                    if (isTv) {
                        NavigationRail(
                            modifier = Modifier.fillMaxHeight()
                        ) {
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
                        modifier = Modifier.weight(1f).fillMaxHeight()
                    ) {
                    composable(Screen.Gallery.route) {
                        GalleryScreen(
                            onNavigateToFolder = { /* Handle folder navigation */ },
                            onFileClick = { file ->
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
                            viewModel = albumViewModel,
                            onShowMediaViewer = {
                                showBottomNav = false
                            },
                            onHideMediaViewer = {
                                showBottomNav = true
                            },
                            modifier = Modifier.padding(bottom = 60.dp)
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

                    composable(Screen.Upload.route) {
                        UploadScreen(
                            onNavigateBack = { navController.popBackStack() },
                            viewModel = uploadViewModel
                        )
                    }
                }
            }
        }
    }
    
    // 自定义顶部栏（包含状态栏和导航栏）
    Column(
            modifier = Modifier.fillMaxWidth()
        ) {
            // 状态栏背景（随滚动变化透明度）- 与导航栏同色
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .alpha(topBarAlpha)
                    .background(MaterialTheme.colorScheme.surface)
                    .windowInsetsTopHeight(WindowInsets.statusBars)
            )
            
            // 顶部导航栏 - 紧凑高度，根据当前页面显示不同操作
            Surface(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(48.dp),
                color = MaterialTheme.colorScheme.surface.copy(alpha = topBarAlpha),
                tonalElevation = 0.dp,
                shadowElevation = 0.dp
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(horizontal = 16.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        "HomeDrive",
                        style = MaterialTheme.typography.titleMedium,
                        modifier = Modifier
                            .weight(1f)
                            .alpha(titleAlpha)
                    )
                    
                    // 根据当前页面显示不同的操作按钮
                    when (currentRoute) {
                        Screen.Albums.route -> {
                            // 相册页面：创建相册 + 设置（只有一项操作，直接点击触发）
                            IconButton(
                                onClick = { albumViewModel.showCreateDialog() },
                                modifier = Modifier.size(40.dp)
                            ) {
                                Icon(
                                    Icons.Default.Add,
                                    contentDescription = "创建相册",
                                    modifier = Modifier.size(22.dp)
                                )
                            }
                            IconButton(
                                onClick = { navController.navigate(Screen.Settings.route) },
                                modifier = Modifier.size(40.dp)
                            ) {
                                Icon(
                                    Icons.Default.Settings,
                                    contentDescription = "设置",
                                    modifier = Modifier.size(22.dp)
                                )
                            }
                        }
                        else -> {
                            // 其他页面（首页）：上传 + 设置
                            IconButton(
                                onClick = { navController.navigate(Screen.Upload.route) },
                                modifier = Modifier.size(40.dp)
                            ) {
                                Icon(
                                    Icons.Default.Add,
                                    contentDescription = "上传",
                                    modifier = Modifier.size(22.dp)
                                )
                            }
                            IconButton(
                                onClick = { navController.navigate(Screen.Settings.route) },
                                modifier = Modifier.size(40.dp)
                            ) {
                                Icon(
                                    Icons.Default.Settings,
                                    contentDescription = "设置",
                                    modifier = Modifier.size(22.dp)
                                )
                            }
                        }
                    }
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
fun getStringResource(id: Int): String {
    return when (id) {
        R.string.gallery -> "图库"
        R.string.albums -> "图集"
        R.string.shares -> "分享"
        R.string.settings -> "设置"
        R.string.upload -> "上传"
        else -> ""
    }
}

/**
 * 悬浮式底部导航栏 - 紧凑文字样式
 */
@Composable
fun FloatingBottomNavBar(
    items: List<Screen>,
    currentDestination: androidx.navigation.NavDestination?,
    onNavigate: (Screen) -> Unit
) {
    // 在暗黑模式下使用更浅的背景色，以便与 app 背景区分开
    val navBarColor = if (androidx.compose.foundation.isSystemInDarkTheme()) {
        MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.95f)
    } else {
        MaterialTheme.colorScheme.surface.copy(alpha = 0.95f)
    }
    
    Surface(
        modifier = Modifier
            .padding(bottom = 16.dp)
            .height(44.dp)
            .fillMaxWidth(0.45f),
        shape = RoundedCornerShape(22.dp),
        color = navBarColor,
        tonalElevation = 8.dp,
        shadowElevation = 8.dp
    ) {
        Row(
            modifier = Modifier
                .padding(horizontal = 6.dp, vertical = 4.dp)
                .fillMaxHeight(),
            horizontalArrangement = Arrangement.spacedBy(4.dp, Alignment.CenterHorizontally),
            verticalAlignment = Alignment.CenterVertically
        ) {
            items.forEach { screen ->
                val isSelected = currentDestination?.hierarchy?.any { it.route == screen.route } == true
                
                val backgroundColor by animateColorAsState(
                    targetValue = if (isSelected) MaterialTheme.colorScheme.primary.copy(alpha = 0.15f)
                                  else Color.Transparent,
                    animationSpec = tween(durationMillis = 200),
                    label = "bgColor"
                )
                
                val contentColor by animateColorAsState(
                    targetValue = if (isSelected) MaterialTheme.colorScheme.primary
                                  else MaterialTheme.colorScheme.onSurfaceVariant,
                    animationSpec = tween(durationMillis = 200),
                    label = "contentColor"
                )
                
                Surface(
                    modifier = Modifier
                        .height(36.dp)
                        .weight(1f),
                    shape = RoundedCornerShape(18.dp),
                    color = backgroundColor,
                    onClick = { onNavigate(screen) }
                ) {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(horizontal = 12.dp),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(
                            text = getStringResource(screen.title),
                            style = MaterialTheme.typography.labelLarge,
                            color = contentColor,
                            maxLines = 1
                        )
                    }
                }
            }
        }
    }
}
