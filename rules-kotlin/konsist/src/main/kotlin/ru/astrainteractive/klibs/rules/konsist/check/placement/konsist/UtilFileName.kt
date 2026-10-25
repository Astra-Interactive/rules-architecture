package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.baseName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.simpleName
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a file named after nothing: in a `util` package, a file whose functions all extend one receiver type is
 * named `<Receiver>Ext.kt`; and a file named `Utils.kt`, `Helpers.kt`, `Extensions.kt` or the like is reported
 * anywhere.
 */
internal class UtilFileName : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun findExtensionFileMessage(file: KoFileDeclaration): String? {
        val functions = file.functions(includeNested = false, includeLocal = false)
        if (functions.isEmpty() || functions.any { function -> function.receiverType == null }) return null
        val receivers = functions.mapNotNull { function -> function.receiverType?.simpleName }.toSet()
        val receiver = receivers.singleOrNull() ?: return null
        val expectedName = receiver + EXT_SUFFIX
        if (file.baseName == expectedName) return null
        return "Rename ${file.baseName}.kt to $expectedName.kt: a util file of $receiver extensions is named after " +
            "its receiver."
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles.mapNotNull { file ->
            val message = if (file.baseName in VAGUE_NAMES) {
                "Rename ${file.baseName}.kt after what its functions do, or <Subject>Ext.kt for extensions."
            } else if (sources.placementOf(file)?.kind == Kind.UTIL) {
                findExtensionFileMessage(file)
            } else {
                null
            }
            message?.let { text ->
                Violation(
                    RULE,
                    sources.relativePath(file),
                    file.baseName,
                    text
                )
            }
        }
    }

    private companion object {
        const val RULE = "UtilFileName"
        const val EXT_SUFFIX = "Ext"

        @JvmField
        val VAGUE_NAMES = setOf(
            "Utils",
            "Util",
            "Helpers",
            "Helper",
            "Extensions",
            "Ext",
            "Misc"
        )
    }
}
