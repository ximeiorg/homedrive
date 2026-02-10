package com.kingzcheung.homedrive.data.local

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.*
import androidx.datastore.preferences.preferencesDataStore
import com.google.gson.Gson
import com.kingzcheung.homedrive.data.model.Member
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map

private val Context.dataStore: DataStore<Preferences> by preferencesDataStore(name = "homedrive_prefs")

class PreferencesManager(context: Context) {
    private val dataStore = context.dataStore
    private val gson = Gson()

    companion object {
        private val SERVER_URL = stringPreferencesKey("server_url")
        private val TOKEN = stringPreferencesKey("token")
        private val MEMBER_JSON = stringPreferencesKey("member_json")
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

    // 获取持久化的会员信息
    val member: Flow<Member?> = dataStore.data.map { preferences ->
        preferences[MEMBER_JSON]?.let { json ->
            try {
                gson.fromJson(json, Member::class.java)
            } catch (e: Exception) {
                null
            }
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

    // 保存完整的会员信息
    suspend fun setMember(member: Member) {
        dataStore.edit { preferences ->
            preferences[MEMBER_JSON] = gson.toJson(member)
        }
    }

    suspend fun setFirstLaunch(isFirst: Boolean) {
        dataStore.edit { preferences ->
            preferences[IS_FIRST_LAUNCH] = isFirst
        }
    }

    // 清除所有持久化数据（退出登录）
    suspend fun clearAll() {
        dataStore.edit { preferences ->
            preferences.clear()
        }
    }
}
