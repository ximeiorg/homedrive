package com.kingzcheung.homedrive.data.local

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.*
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map

private val Context.dataStore: DataStore<Preferences> by preferencesDataStore(name = "homedrive_prefs")

class PreferencesManager(context: Context) {
    private val dataStore = context.dataStore

    companion object {
        private val SERVER_URL = stringPreferencesKey("server_url")
        private val TOKEN = stringPreferencesKey("token")
        private val USERNAME = stringPreferencesKey("username")
        private val USER_ID = longPreferencesKey("user_id")
        private val IS_FIRST_LAUNCH = booleanPreferencesKey("is_first_launch")
    }

    val serverUrl: Flow<String> = dataStore.data.map { preferences ->
        preferences[SERVER_URL] ?: ""
    }

    val token: Flow<String?> = dataStore.data.map { preferences ->
        preferences[TOKEN]
    }

    val isLoggedIn: Flow<Boolean> = dataStore.data.map { preferences ->
        preferences[TOKEN] != null
    }

    val userInfo: Flow<UserInfo?> = dataStore.data.map { preferences ->
        if (preferences[TOKEN] != null) {
            UserInfo(
                username = preferences[USERNAME] ?: "",
                userId = preferences[USER_ID] ?: 0L
            )
        } else {
            null
        }
    }

    val isFirstLaunch: Flow<Boolean> = dataStore.data.map { preferences ->
        preferences[IS_FIRST_LAUNCH] ?: true
    }

    suspend fun getTokenSync(): String? {
        var token: String? = null
        dataStore.data.collect { preferences ->
            token = preferences[TOKEN]
        }
        return token
    }

    suspend fun setServerUrl(url: String) {
        dataStore.edit { preferences ->
            preferences[SERVER_URL] = url
        }
    }

    suspend fun setToken(token: String) {
        dataStore.edit { preferences ->
            preferences[TOKEN] = token
        }
    }

    suspend fun setUserInfo(username: String, userId: Long) {
        dataStore.edit { preferences ->
            preferences[USERNAME] = username
            preferences[USER_ID] = userId
        }
    }

    suspend fun setFirstLaunch(isFirst: Boolean) {
        dataStore.edit { preferences ->
            preferences[IS_FIRST_LAUNCH] = isFirst
        }
    }

    suspend fun clearAll() {
        dataStore.edit { preferences ->
            preferences.clear()
        }
    }
}

data class UserInfo(
    val username: String,
    val userId: Long
)
