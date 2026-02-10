package com.kingzcheung.homedrive.ui.viewmodel

import android.content.Context
import android.net.Uri
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kingzcheung.homedrive.HomedriveApp
import com.kingzcheung.homedrive.data.api.HomedriveApi
import com.kingzcheung.homedrive.data.local.PreferencesManager
import com.kingzcheung.homedrive.data.repository.UploadRepository
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
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

    companion object {
        private const val TAG = "HomeDrive_Upload"
    }

    private val repository = UploadRepository(api)
    private val _uiState = MutableStateFlow(UploadUiState())
    val uiState: StateFlow<UploadUiState> = _uiState.asStateFlow()

    fun addFiles(uris: List<Uri>, context: Context) {
        Log.d(TAG, "addFiles called with ${uris.size} URIs")
        
        val newFiles = uris.mapNotNull { uri ->
            val cursor = context.contentResolver.query(uri, null, null, null, null)
            cursor?.use {
                if (it.moveToFirst()) {
                    val nameIndex = it.getColumnIndex(android.provider.OpenableColumns.DISPLAY_NAME)
                    val sizeIndex = it.getColumnIndex(android.provider.OpenableColumns.SIZE)
                    val name = if (nameIndex >= 0) it.getString(nameIndex) else uri.lastPathSegment ?: "unknown"
                    val size = if (sizeIndex >= 0) it.getLong(sizeIndex) else 0L
                    val type = context.contentResolver.getType(uri) ?: "application/octet-stream"

                    Log.d(TAG, "File added: name=$name, size=$size, type=$type")
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
        
        Log.d(TAG, "Total files in list: ${_uiState.value.files.size}")
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

    fun startUpload() {
        Log.d(TAG, "startUpload called")
        Log.d(TAG, "Current state: isUploading=${_uiState.value.isUploading}, files=${_uiState.value.files.size}")
        
        val pendingFiles = _uiState.value.files.filter { it.state == UploadState.PENDING }
        Log.d(TAG, "Pending files count: ${pendingFiles.size}")
        
        if (pendingFiles.isEmpty()) {
            Log.w(TAG, "No pending files to upload")
            return
        }
        
        if (_uiState.value.isUploading) {
            Log.w(TAG, "Already uploading, skipping")
            return
        }

        viewModelScope.launch {
            Log.i(TAG, "=== Starting upload process ===")
            Log.i(TAG, "Files to upload: ${pendingFiles.size}")
            
            _uiState.value = _uiState.value.copy(
                isUploading = true,
                currentFileIndex = 0,
                uploadProgress = 0,
                successCount = 0,
                errorCount = 0
            )

            var successCount = 0
            var errorCount = 0
            val totalFiles = _uiState.value.files.size

            _uiState.value.files.forEachIndexed { index, uploadFile ->
                if (uploadFile.state == UploadState.PENDING) {
                    Log.i(TAG, "--- Uploading file ${index + 1}/$totalFiles: ${uploadFile.name} ---")
                    _uiState.value = _uiState.value.copy(currentFileIndex = index)

                    // Update current file to uploading
                    updateFileState(index, UploadState.UPLOADING)

                    try {
                        Log.d(TAG, "Calling uploadFileFromUri for: ${uploadFile.name}")
                        val result = uploadFileFromUri(uploadFile.uri, uploadFile.name)
                        
                        result.onSuccess {
                            Log.i(TAG, "✓ File ${uploadFile.name} uploaded successfully")
                            updateFileState(index, UploadState.SUCCESS)
                            successCount++
                        }.onFailure { e ->
                            Log.e(TAG, "✗ File ${uploadFile.name} upload failed: ${e.message}", e)
                            updateFileState(index, UploadState.ERROR)
                            errorCount++
                        }
                    } catch (e: Exception) {
                        Log.e(TAG, "✗ Exception during upload of ${uploadFile.name}: ${e.message}", e)
                        updateFileState(index, UploadState.ERROR)
                        errorCount++
                    }

                    _uiState.value = _uiState.value.copy(
                        uploadProgress = ((index + 1) * 100) / totalFiles,
                        successCount = successCount,
                        errorCount = errorCount
                    )
                    
                    Log.d(TAG, "Progress updated: ${_uiState.value.uploadProgress}%, success=$successCount, error=$errorCount")
                }
            }

            Log.i(TAG, "=== Upload completed ===")
            Log.i(TAG, "Results: success=$successCount, error=$errorCount")
            
            _uiState.value = _uiState.value.copy(
                isUploading = false
            )
        }
    }

    private suspend fun uploadFileFromUri(uri: Uri, fileName: String): Result<com.kingzcheung.homedrive.data.model.UploadResponse> {
        return withContext(Dispatchers.IO) {
            try {
                Log.d(TAG, "[IO] Creating temp file for: $fileName")
                val tempFile = createTempFileFromUri(uri, fileName)
                if (tempFile == null) {
                    Log.e(TAG, "[IO] Failed to create temp file for: $fileName")
                    return@withContext Result.failure(Exception("Cannot read file from URI"))
                }
                
                Log.d(TAG, "[IO] Temp file created: ${tempFile.absolutePath}")
                Log.d(TAG, "[IO] Temp file size: ${tempFile.length()} bytes")
                
                Log.d(TAG, "[IO] Calling repository.uploadFile...")
                val result = repository.uploadFile(tempFile)
                
                // Clean up temp file
                if (tempFile.delete()) {
                    Log.d(TAG, "[IO] Temp file deleted")
                } else {
                    Log.w(TAG, "[IO] Failed to delete temp file")
                }
                
                result.onSuccess {
                    Log.d(TAG, "[IO] Upload API call successful")
                }.onFailure { e ->
                    Log.e(TAG, "[IO] Upload API call failed: ${e.message}")
                }
                
                result
            } catch (e: Exception) {
                Log.e(TAG, "[IO] Exception in uploadFileFromUri: ${e.message}", e)
                Result.failure(e)
            }
        }
    }

    private fun createTempFileFromUri(uri: Uri, fileName: String): File? {
        return try {
            val context = HomedriveApp.instance
            val tempDir = context.cacheDir
            
            // Create unique temp file name
            val uniqueFileName = "upload_${System.currentTimeMillis()}_$fileName"
            val tempFile = File(tempDir, uniqueFileName)
            
            Log.d(TAG, "[TempFile] Creating: ${tempFile.absolutePath}")
            
            context.contentResolver.openInputStream(uri)?.use { inputStream ->
                FileOutputStream(tempFile).use { outputStream ->
                    val bytesCopied = inputStream.copyTo(outputStream)
                    Log.d(TAG, "[TempFile] Copied $bytesCopied bytes")
                }
            }
            
            if (tempFile.exists() && tempFile.length() > 0) {
                Log.d(TAG, "[TempFile] Success: ${tempFile.length()} bytes")
                tempFile
            } else {
                Log.e(TAG, "[TempFile] File is empty or doesn't exist")
                tempFile.delete()
                null
            }
        } catch (e: Exception) {
            Log.e(TAG, "[TempFile] Error: ${e.message}", e)
            null
        }
    }

    private fun updateFileState(index: Int, state: UploadState) {
        val newFiles = _uiState.value.files.toMutableList()
        if (index in newFiles.indices) {
            newFiles[index] = newFiles[index].copy(state = state)
            _uiState.value = _uiState.value.copy(files = newFiles)
            Log.d(TAG, "File state updated: index=$index, state=$state")
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
