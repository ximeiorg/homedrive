package com.kingzcheung.homedrive.data.network

import android.content.Context
import android.net.wifi.WifiManager
import android.util.Log
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.withContext
import java.net.HttpURLConnection
import java.net.Inet4Address
import java.net.InetAddress
import java.net.NetworkInterface
import java.net.URL

/**
 * 表示扫描发现的服务器
 */
data class DiscoveredServer(
    val ipAddress: String,
    val port: Int = 2300,
    val displayName: String = "$ipAddress:$port"
) {
    val fullUrl: String
        get() = "http://$ipAddress:$port"
}

/**
 * 网络接口信息
 */
data class NetworkInterfaceInfo(
    val name: String,
    val ipAddress: String,
    val networkPrefix: String,
    val subnetMask: Int // 子网掩码位数，如 24 表示 255.255.255.0
)

/**
 * 网络扫描器，用于扫描局域网中运行在指定端口的服务器
 */
class NetworkScanner(private val context: Context) {
    
    companion object {
        private const val TAG = "NetworkScanner"
        private const val DEFAULT_PORT = 2300
        private const val CONNECT_TIMEOUT_MS = 800
        private const val READ_TIMEOUT_MS = 800
        private const val MAX_CONCURRENT_SCANS = 30
    }
    
    /**
     * 获取所有活动的网络接口信息
     */
    fun getActiveNetworkInterfaces(): List<NetworkInterfaceInfo> {
        val interfaces = mutableListOf<NetworkInterfaceInfo>()
        
        try {
            val networkInterfaces = NetworkInterface.getNetworkInterfaces()
            while (networkInterfaces.hasMoreElements()) {
                val networkInterface = networkInterfaces.nextElement()
                
                // 跳过回环接口、未启用的接口、虚拟接口
                if (networkInterface.isLoopback || !networkInterface.isUp) continue
                if (networkInterface.name.startsWith("vnic") || 
                    networkInterface.name.startsWith("docker") ||
                    networkInterface.name.startsWith("veth") ||
                    networkInterface.name.startsWith("tun") ||
                    networkInterface.name.startsWith("ppp")) continue
                
                val addresses = networkInterface.inetAddresses
                while (addresses.hasMoreElements()) {
                    val address = addresses.nextElement()
                    
                    // 只处理 IPv4 地址，跳过回环地址
                    if (address is Inet4Address && !address.isLoopbackAddress) {
                        val ip = address.hostAddress ?: continue
                        
                        // 获取子网掩码
                        val prefixLength = networkInterface.interfaceAddresses
                            .find { it.address == address }
                            ?.networkPrefixLength?.toInt() ?: 24
                        
                        // 计算网段前缀
                        val networkPrefix = getNetworkPrefix(ip, prefixLength)
                        
                        interfaces.add(NetworkInterfaceInfo(
                            name = networkInterface.name,
                            ipAddress = ip,
                            networkPrefix = networkPrefix,
                            subnetMask = prefixLength
                        ))
                        
                        Log.d(TAG, "Found network interface: ${networkInterface.name}, IP: $ip, Prefix: $networkPrefix/$prefixLength")
                    }
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error getting network interfaces", e)
        }
        
        return interfaces
    }
    
    /**
     * 根据本地 IP 地址和子网掩码获取网段前缀
     * 例如：192.168.1.100/24 -> 192.168.1
     */
    private fun getNetworkPrefix(ip: String, prefixLength: Int): String {
        val parts = ip.split(".")
        if (parts.size != 4) return ip
        
        // 对于 /24 子网，返回前三个八位组
        // 对于 /16 子网，返回前两个八位组
        // 对于 /8 子网，返回第一个八位组
        return when {
            prefixLength >= 24 -> "${parts[0]}.${parts[1]}.${parts[2]}"
            prefixLength >= 16 -> "${parts[0]}.${parts[1]}"
            prefixLength >= 8 -> parts[0]
            else -> "${parts[0]}.${parts[1]}.${parts[2]}"
        }
    }
    
    /**
     * 获取当前设备的局域网 IP 地址（返回第一个非回环 IPv4 地址）
     */
    fun getLocalIpAddress(): String? {
        return getActiveNetworkInterfaces().firstOrNull()?.ipAddress
    }
    
    /**
     * 扫描所有活动网络接口所在的网段
     * @param port 要扫描的端口，默认为 2300
     * @param onProgress 进度回调，返回已扫描的数量
     * @return 发现的服务器列表
     */
    suspend fun scanNetwork(
        port: Int = DEFAULT_PORT,
        onProgress: ((scanned: Int, total: Int) -> Unit)? = null
    ): List<DiscoveredServer> = withContext(Dispatchers.IO) {
        val discoveredServers = mutableListOf<DiscoveredServer>()
        val networkInterfaces = getActiveNetworkInterfaces()
        
        if (networkInterfaces.isEmpty()) {
            Log.w(TAG, "No active network interfaces found")
            return@withContext emptyList()
        }
        
        Log.i(TAG, "Starting network scan on ${networkInterfaces.size} interface(s)")
        
        // 计算总共需要扫描的主机数
        val totalHosts = networkInterfaces.sumOf { 
            if (it.subnetMask >= 24) 254 else if (it.subnetMask >= 16) 65534 else 254
        }
        var scannedCount = 0
        
        // 扫描每个网络接口所在的网段
        for (networkInterface in networkInterfaces) {
            Log.i(TAG, "Scanning interface: ${networkInterface.name}, prefix: ${networkInterface.networkPrefix}")
            
            val servers = scanSubnet(
                networkPrefix = networkInterface.networkPrefix,
                port = port,
                onProgress = { scanned ->
                    synchronized(this) {
                        scannedCount += scanned
                        onProgress?.invoke(scannedCount, totalHosts)
                    }
                }
            )
            
            discoveredServers.addAll(servers)
        }
        
        Log.i(TAG, "Network scan completed. Found ${discoveredServers.size} server(s)")
        discoveredServers
    }
    
    /**
     * 扫描指定子网
     */
    private suspend fun scanSubnet(
        networkPrefix: String,
        port: Int,
        onProgress: ((scanned: Int) -> Unit)? = null
    ): List<DiscoveredServer> = coroutineScope {
        val discoveredServers = mutableListOf<DiscoveredServer>()
        
        // 分批扫描，避免创建过多协程
        val results = (1..254).chunked(MAX_CONCURRENT_SCANS).map { batch ->
            batch.map { hostSuffix ->
                async {
                    val hostIp = "$networkPrefix.$hostSuffix"
                    val isServer = checkServer(hostIp, port)
                    
                    onProgress?.invoke(1)
                    
                    if (isServer) {
                        Log.d(TAG, "Found server at $hostIp:$port")
                        DiscoveredServer(hostIp, port)
                    } else {
                        null
                    }
                }
            }.awaitAll()
        }.flatten().filterNotNull()
        
        discoveredServers.addAll(results)
        discoveredServers
    }
    
    /**
     * 检查指定 IP 和端口是否有 HomeDrive 服务器运行
     */
    private fun checkServer(ip: String, port: Int): Boolean {
        return try {
            val url = URL("http://$ip:$port/api/health")
            val connection = url.openConnection() as HttpURLConnection
            connection.apply {
                connectTimeout = CONNECT_TIMEOUT_MS
                readTimeout = READ_TIMEOUT_MS
                requestMethod = "GET"
                instanceFollowRedirects = false
            }
            
            val responseCode = connection.responseCode
            connection.disconnect()
            
            // 如果能收到 200 响应，认为服务器存在
            responseCode == 200
        } catch (e: Exception) {
            false
        }
    }
    
    /**
     * 快速检查单个 IP 是否有服务器运行
     */
    suspend fun checkSingleServer(ip: String, port: Int = DEFAULT_PORT): Boolean {
        return withContext(Dispatchers.IO) {
            checkServer(ip, port)
        }
    }
}
