package ru.astrainteractive.klibs.rules.konsist.layout

/**
 * Where a package sits inside a context: in which module, in which role, and how far below the role.
 *
 * A `null` [role] is the root of the context or of a module, where the use cases live.
 */
internal data class Placement(
    val module: String?,
    val role: Role?,
    val subpackages: List<String>
) {
    /**
     * The technology or external system a `data` or `integration` package is implemented with
     * (`exposed` in `data/exposed`), or `null` at the level of the contract.
     */
    val technology: String?
        get() = subpackages.firstOrNull()?.takeIf { role == Role.DATA || role == Role.INTEGRATION }

    val layer: Layer
        get() = when {
            role == null -> Layer.USE_CASE
            role.isEntryPoint -> Layer.ENTRY_POINT
            role == Role.MODEL -> Layer.MODEL
            role == Role.MAPPING -> Layer.MAPPING
            role == Role.UTIL -> Layer.UTIL
            role == Role.DI -> Layer.DI
            technology == null -> Layer.CONTRACT
            else -> Layer.TECHNOLOGY
        }

    fun describe(): String {
        val path = listOfNotNull(module, role?.segment, technology).joinToString("/")
        return when (layer) {
            Layer.USE_CASE -> if (module == null) "the context root" else "the root of module '$module'"
            Layer.CONTRACT -> "the $path contract"
            else -> "'$path'"
        }
    }
}
