package com.kingzcheung.homedrive.ui.theme

import android.app.Activity
import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat

private val LightColorScheme = lightColorScheme(
    primary = Color(0xFF1976D2),
    onPrimary = Color.White,
    primaryContainer = Color(0xFFBBDEFB),
    onPrimaryContainer = Color(0xFF0D47A1),
    secondary = Color(0xFF43A047),
    onSecondary = Color.White,
    secondaryContainer = Color(0xFFC8E6C9),
    onSecondaryContainer = Color(0xFF1B5E20),
    tertiary = Color(0xFFFFA000),
    onTertiary = Color.White,
    tertiaryContainer = Color(0xFFFFECB3),
    onTertiaryContainer = Color(0xFFE65100),
    error = Color(0xFFD32F2F),
    onError = Color.White,
    errorContainer = Color(0xFFFFCDD2),
    onErrorContainer = Color(0xFFB71C1C),
    background = Color(0xFFFAFAFA),
    onBackground = Color(0xFF212121),
    surface = Color.White,
    onSurface = Color(0xFF212121),
    surfaceVariant = Color(0xFFE0E0E0),
    onSurfaceVariant = Color(0xFF616161),
    outline = Color(0xFFBDBDBD),
    outlineVariant = Color(0xFFE0E0E0),
    inverseSurface = Color(0xFF303030),
    inverseOnSurface = Color(0xFFFAFAFA),
    inversePrimary = Color(0xFF90CAF9)
)

private val DarkColorScheme = darkColorScheme(
    primary = Color(0xFF90CAF9),
    onPrimary = Color(0xFF0D47A1),
    primaryContainer = Color(0xFF1565C0),
    onPrimaryContainer = Color(0xFFBBDEFB),
    secondary = Color(0xFF81C784),
    onSecondary = Color(0xFF1B5E20),
    secondaryContainer = Color(0xFF2E7D32),
    onSecondaryContainer = Color(0xFFC8E6C9),
    tertiary = Color(0xFFFFCC80),
    onTertiary = Color(0xFFE65100),
    tertiaryContainer = Color(0xFFF57C00),
    onTertiaryContainer = Color(0xFFFFECB3),
    error = Color(0xFFEF9A9A),
    onError = Color(0xFFB71C1C),
    errorContainer = Color(0xFFC62828),
    onErrorContainer = Color(0xFFFFCDD2),
    background = Color(0xFF121212),
    onBackground = Color(0xFFE0E0E0),
    surface = Color(0xFF1E1E1E),
    onSurface = Color(0xFFE0E0E0),
    surfaceVariant = Color(0xFF2C2C2C),
    onSurfaceVariant = Color(0xFFB0B0B0),
    outline = Color(0xFF5C5C5C),
    outlineVariant = Color(0xFF424242),
    inverseSurface = Color(0xFFE0E0E0),
    inverseOnSurface = Color(0xFF121212),
    inversePrimary = Color(0xFF1976D2)
)

@Composable
fun HomedriveTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    dynamicColor: Boolean = true,
    content: @Composable () -> Unit
) {
    val colorScheme = when {
        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
            val context = LocalContext.current
            if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        }
        darkTheme -> DarkColorScheme
        else -> LightColorScheme
    }

    val view = LocalView.current
    if (!view.isInEditMode) {
        SideEffect {
            val window = (view.context as Activity).window
            // 状态栏透明，让内容延伸到状态栏区域
            window.statusBarColor = android.graphics.Color.TRANSPARENT
            
            val windowInsetsController = WindowCompat.getInsetsController(window, view)
            // isAppearanceLightStatusBars: 
            // - true = 状态栏内容使用深色（适用于浅色背景）
            // - false = 状态栏内容使用浅色（适用于深色背景）
            // 在暗黑模式下，背景是深色的，需要浅色状态栏内容，所以设置为 false
            // 在亮色模式下，背景是浅色的，需要深色状态栏内容，所以设置为 true
            windowInsetsController.isAppearanceLightStatusBars = !darkTheme
            windowInsetsController.isAppearanceLightNavigationBars = !darkTheme
        }
    }

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content
    )
}
