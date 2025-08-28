/**
 * Compatibility functions for buildList, ensuring support for Kotlin versions < 1.6.0
 *
 * These functions provide the same functionality as the standard library buildList functions
 * introduced in Kotlin 1.6.0. On Kotlin 1.6+, the compiler will prefer the standard library
 * versions due to better overload resolution, so these serve as fallbacks for older versions.
 *
 * The functions are inline and generate efficient bytecode equivalent to the standard library
 * implementations, so there's no performance penalty when included.
 */
inline fun <T> buildList(capacity: Int, builderAction: MutableList<T>.() -> Unit): List<T> {
    val list = ArrayList<T>(capacity)
    list.builderAction()
    return list
}

inline fun <T> buildList(builderAction: MutableList<T>.() -> Unit): List<T> {
    val list = mutableListOf<T>()
    list.builderAction()
    return list
}
