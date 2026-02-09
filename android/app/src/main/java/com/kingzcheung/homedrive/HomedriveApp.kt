package com.kingzcheung.homedrive

import android.app.Application
import com.kingzcheung.homedrive.di.AppContainer

class HomedriveApp : Application() {
    override fun onCreate() {
        super.onCreate()
        AppContainer.initialize(this)
    }
}
