package godau.fynn.usagedirectplus.wrapper

/** JNI boundary for the Rust usage aggregation core. */
internal object RustUsageCore {
    private val isAvailable: Boolean = runCatching {
        System.loadLibrary("usage_core")
        true
    }.getOrDefault(false)

    fun aggregate(
        beginTimes: LongArray,
        endTimes: LongArray,
        appIndexes: IntArray,
        appCount: Int
    ): LongArray? {
        if (!isAvailable) return null
        return runCatching {
            nativeAggregate(beginTimes, endTimes, appIndexes, appCount)
        }.getOrNull()
    }

    @JvmStatic
    private external fun nativeAggregate(
        beginTimes: LongArray,
        endTimes: LongArray,
        appIndexes: IntArray,
        appCount: Int
    ): LongArray
}
