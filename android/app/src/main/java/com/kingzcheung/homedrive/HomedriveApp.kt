package com.kingzcheung.homedrive

import android.app.Application
import android.content.Context
import com.kingzcheung.homedrive.di.AppContainer

class HomedriveApp : Application() {
    
    companion object {
        lateinit var instance: HomedriveApp
            private set
    }
    
    override fun onCreate() {
        super.onCreate()
        instance = this
        AppContainer.initialize(this)
    }
}
