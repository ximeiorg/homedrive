package com.kingzcheung.homedrive.ui.screen

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.kingzcheung.homedrive.R
import com.kingzcheung.homedrive.data.model.Album
import com.kingzcheung.homedrive.data.model.FileItem
import com.kingzcheung.homedrive.ui.tv.TvAlbumCard
import com.kingzcheung.homedrive.ui.tv.TvGalleryGrid
import com.kingzcheung.homedrive.ui.tv.TvHomeBanner
import com.kingzcheung.homedrive.ui.viewmodel.AlbumViewModel
import com.kingzcheung.homedrive.ui.viewmodel.GalleryViewModel
import com.kingzcheung.homedrive.ui.viewmodel.SettingsViewModel
import com.kingzcheung.homedrive.ui.viewmodel.ShareViewModel

/**
 * TV 优化的主屏幕
 * 使用侧边导航和更大的 UI 元素
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeTvScreen(
    onNavigateToGallery: () -> Unit,
    onNavigateToAlbums: () -> Unit,
    onNavigateToShares: () -> Unit,
    onNavigateToSettings: () -> Unit,
    onFileClick: (FileItem) -> Unit,
    onAlbumClick: (Album) -> Unit,
    galleryViewModel: GalleryViewModel,
    albumViewModel: AlbumViewModel,
    shareViewModel: ShareViewModel,
    settingsViewModel: SettingsViewModel
) {
    val configuration = LocalConfiguration.current
    val isTv = configuration.screenWidthDp >= 840

    if (isTv) {
        TvMainLayout(
            onNavigateToGallery = onNavigateToGallery,
            onNavigateToAlbums = onNavigateToAlbums,
            onNavigateToShares = onNavigateToShares,
            onNavigateToSettings = onNavigateToSettings,
            onFileClick = onFileClick,
            onAlbumClick = onAlbumClick,
            galleryViewModel = galleryViewModel,
            albumViewModel = albumViewModel,
            shareViewModel = shareViewModel,
            settingsViewModel = settingsViewModel
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun TvMainLayout(
    onNavigateToGallery: () -> Unit,
    onNavigateToAlbums: () -> Unit,
    onNavigateToShares: () -> Unit,
    onNavigateToSettings: () -> Unit,
    onFileClick: (FileItem) -> Unit,
    onAlbumClick: (Album) -> Unit,
    galleryViewModel: GalleryViewModel,
    albumViewModel: AlbumViewModel,
    shareViewModel: ShareViewModel,
    settingsViewModel: SettingsViewModel
) {
    var selectedTab by remember { mutableStateOf(0) }
    val tabs = listOf(
        Triple(0, stringResource(R.string.gallery), Icons.Default.PhotoLibrary),
        Triple(1, stringResource(R.string.albums), Icons.Default.Collections),
        Triple(2, stringResource(R.string.shares), Icons.Default.Share),
        Triple(3, stringResource(R.string.settings), Icons.Default.Settings)
    )

    Row(modifier = Modifier.fillMaxSize()) {
        // 左侧导航
        Column(
            modifier = Modifier
                .width(240.dp)
                .fillMaxHeight()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            // Logo/标题
            Text(
                text = stringResource(R.string.app_name),
                style = MaterialTheme.typography.headlineMedium,
                modifier = Modifier.padding(bottom = 32.dp)
            )

            // 导航项
            tabs.forEach { (index, title, icon) ->
                val isSelected = selectedTab == index
                NavigationRailItem(
                    icon = { Icon(icon, contentDescription = null) },
                    label = {
                        Text(
                            text = title,
                            style = MaterialTheme.typography.titleMedium
                        )
                    },
                    selected = isSelected,
                    onClick = {
                        selectedTab = index
                        when (index) {
                            0 -> onNavigateToGallery()
                            1 -> onNavigateToAlbums()
                            2 -> onNavigateToShares()
                            3 -> onNavigateToSettings()
                        }
                    },
                    modifier = Modifier.height(64.dp)
                )
            }
        }

        // 右侧内容区域
        Box(
            modifier = Modifier
                .weight(1f)
                .fillMaxHeight()
        ) {
            when (selectedTab) {
                0 -> TvGalleryContent(
                    galleryViewModel = galleryViewModel,
                    onFileClick = onFileClick
                )
                1 -> TvAlbumsContent(
                    albumViewModel = albumViewModel,
                    onAlbumClick = onAlbumClick
                )
                2 -> ShareScreen(onNavigateBack = { }, viewModel = shareViewModel)
                3 -> SettingsScreen(
                    onNavigateBack = { },
                    onLogout = { },
                    viewModel = settingsViewModel
                )
            }
        }
    }
}

@Composable
private fun TvGalleryContent(
    galleryViewModel: GalleryViewModel,
    onFileClick: (FileItem) -> Unit
) {
    val uiState by galleryViewModel.uiState.collectAsState()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp)
    ) {
        // 标题栏
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = stringResource(R.string.gallery),
                style = MaterialTheme.typography.headlineMedium
            )
            Row {
                IconButton(onClick = { /* TODO: Refresh */ }) {
                    Icon(Icons.Default.Refresh, contentDescription = stringResource(R.string.refresh))
                }
                IconButton(onClick = { /* Search */ }) {
                    Icon(Icons.Default.Search, contentDescription = stringResource(R.string.search))
                }
            }
        }

        Spacer(modifier = Modifier.height(16.dp))

        // 内容
        when {
            uiState.files.isEmpty() -> {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Icon(
                            Icons.Default.FolderOpen,
                            contentDescription = null,
                            modifier = Modifier.size(64.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.height(16.dp))
                        Text(
                            text = stringResource(R.string.empty_folder),
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }
            else -> {
                TvGalleryGrid(
                    files = uiState.files,
                    onFileClick = onFileClick,
                    modifier = Modifier.fillMaxSize()
                )
            }
        }
    }
}

@Composable
private fun TvAlbumsContent(
    albumViewModel: AlbumViewModel,
    onAlbumClick: (Album) -> Unit
) {
    val uiState by albumViewModel.uiState.collectAsState()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp)
    ) {
        // 标题栏
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = stringResource(R.string.albums),
                style = MaterialTheme.typography.headlineMedium
            )
            IconButton(onClick = { albumViewModel.loadAlbums() }) {
                Icon(Icons.Default.Refresh, contentDescription = stringResource(R.string.refresh))
            }
        }

        Spacer(modifier = Modifier.height(16.dp))

        // 内容
        when {
            uiState.isLoading && uiState.albums.isEmpty() -> {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    CircularProgressIndicator()
                }
            }
            uiState.albums.isEmpty() -> {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Icon(
                            Icons.Default.PhotoLibrary,
                            contentDescription = null,
                            modifier = Modifier.size(64.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.height(16.dp))
                        Text(
                            text = stringResource(R.string.empty_albums),
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }
            else -> {
                LazyVerticalGrid(
                    columns = GridCells.Adaptive(minSize = 280.dp),
                    horizontalArrangement = Arrangement.spacedBy(16.dp),
                    verticalArrangement = Arrangement.spacedBy(16.dp),
                    modifier = Modifier.fillMaxSize()
                ) {
                    items(uiState.albums, key = { it.id }) { album ->
                        TvAlbumCard(
                            album = album,
                            onClick = { onAlbumClick(album) }
                        )
                    }
                }
            }
        }
    }
}
