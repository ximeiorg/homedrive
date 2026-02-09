package com.kingzcheung.homedrive.ui.viewmodel

import android.content.Context
import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kingzcheung.homedrive.data.api.HomedriveApi
import com.kingzcheung.homedrive.data.local.PreferencesManager
import com.kingzcheung.homedrive.data.repository.UploadRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import java.io.File
import java.io.FileOutputStream

data class UploadFile(
    val uri: Uri,
    val name: String,
    val size: Long,
    val type: String,
    val progress: Int = 0,
    val state: UploadState = UploadState.PENDING
)

enum class UploadState {
    PENDING,
    UPLOADING,
    SUCCESS,
    ERROR
}

data class UploadUiState(
    val files: List<UploadFile> = emptyList(),
    val isUploading: Boolean = false,
    val uploadProgress: Int = 0,
    val currentFileIndex: Int = 0,
    val error: String? = null,
    val successCount: Int = 0,
    val errorCount: Int = 0
)

class UploadViewModel(
    private val api: HomedriveApi,
    private val preferencesManager: PreferencesManager
) : ViewModel() {

    private val repository = UploadRepository(api)
    private val _uiState = MutableStateFlow(UploadUiState())
    val uiState: StateFlow<UploadUiState> = _uiState.asStateFlow()

    fun addFiles(uris: List<Uri>, context: Context) {
        val newFiles = uris.mapNotNull { uri ->
            val cursor = context.contentResolver.query(uri, null, null, null, null)
            cursor?.use {
                if (it.moveToFirst()) {
                    val nameIndex = it.getColumnIndex(android.provider.OpenableColumns.DISPLAY_NAME)
                    val sizeIndex = it.getColumnIndex(android.provider.OpenableColumns.SIZE)
                    val name = if (nameIndex >= 0) it.getString(nameIndex) else uri.lastPathSegment ?: "unknown"
                    val size = if (sizeIndex >= 0) it.getLong(sizeIndex) else 0L
                    val type = context.contentResolver.getType(uri) ?: "application/octet-stream"

                    UploadFile(uri, name, size, type)
                } else null
            } ?: run {
                val name = uri.lastPathSegment ?: "unknown"
                UploadFile(uri, name, 0L, context.contentResolver.getType(uri) ?: "application/octet-stream")
            }
        }

        _uiState.value = _uiState.value.copy(
            files = _uiState.value.files + newFiles
        )
    }

    fun removeFile(index: Int) {
        val newFiles = _uiState.value.files.toMutableList()
        if (index in newFiles.indices) {
            newFiles.removeAt(index)
            _uiState.value = _uiState.value.copy(files = newFiles)
        }
    }

    fun clearAllFiles() {
        _uiState.value = _uiState.value.copy(files = emptyList())
    }

    fun startUpload(path: String = "/") {
        val pendingFiles = _uiState.value.files.filter { it.state == UploadState.PENDING }
        if (pendingFiles.isEmpty() || _uiState.value.isUploading) return

        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isUploading = true,
                currentFileIndex = 0,
                uploadProgress = 0,
                successCount = 0,
                errorCount = 0
            )

            var successCount = 0
            var errorCount = 0

            _uiState.value.files.forEachIndexed { index, uploadFile ->
                if (uploadFile.state == UploadState.PENDING) {
                    _uiState.value = _uiState.value.copy(currentFileIndex = index)

                    // Update current file to uploading
                    updateFileState(index, UploadState.UPLOADING)

                    val result = uploadFileFromUri(uploadFile.uri, path)
                    result.onSuccess {
                        updateFileState(index, UploadState.SUCCESS)
                        successCount++
                    }.onFailure {
                        updateFileState(index, UploadState.ERROR)
                        errorCount++
                    }

                    _uiState.value = _uiState.value.copy(
                        uploadProgress = ((index + 1) * 100) / _uiState.value.files.size,
                        successCount = successCount,
                        errorCount = errorCount
                    )
                }
            }

            _uiState.value = _uiState.value.copy(
                isUploading = false
            )
        }
    }

    private suspend fun uploadFileFromUri(uri: Uri, path: String): Result<com.kingzcheung.homedrive.data.model.UploadResponse> {
        val tempFile = createTempFileFromUri(uri) ?: return Result.failure(Exception("Cannot read file"))

        return repository.uploadFile(tempFile, path).also {
            tempFile.delete()
        }
    }

    private fun createTempFileFromUri(uri: Uri): File? {
        return try {
            val context = com.kingzcheung.homedrive.HomedriveApp::class.java
            null
        } catch (e: Exception) {
            null
        }
    }

    private fun updateFileState(index: Int, state: UploadState) {
        val newFiles = _uiState.value.files.toMutableList()
        if (index in newFiles.indices) {
            newFiles[index] = newFiles[index].copy(state = state)
            _uiState.value = _uiState.value.copy(files = newFiles)
        }
    }

    fun retryFailed() {
        val newFiles = _uiState.value.files.map { file ->
            if (file.state == UploadState.ERROR) {
                file.copy(state = UploadState.PENDING, progress = 0)
            } else file
        }
        _uiState.value = _uiState.value.copy(files = newFiles)
    }

    fun clearError() {
        _uiState.value = _uiState.value.copy(error = null)
    }
}
