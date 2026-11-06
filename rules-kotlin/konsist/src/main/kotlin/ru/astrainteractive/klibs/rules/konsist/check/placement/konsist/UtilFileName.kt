package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.baseName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.typePath
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind

/**
 * Reports a file named after nothing: in a `util` package, a file whose functions all extend one receiver type is
 * named `<Receiver>Ext.kt`, a nested receiver after its outermost type (`TagDTOExt.kt` for `TagDTO.Real`), its full
 * name (`TagDTORealExt.kt`) or its own (`RealExt.kt`); and a file named `Utils.kt`, `Helpers.kt`, `Extensions.kt` or
 * the like is reported anywhere.
 */
internal class UtilFileName : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun findExtensionFileMessage(file: KoFileDeclaration): String? {
        val functions = file.functions(includeNested = false, includeLocal = false)
        if (functions.isEmpty() || functions.any { function -> function.receiverType == null }) return null
        val receivers = functions.mapNotNull { function -> function.receiverType?.typePath }.toSet()
        val receiver = receivers.singleOrNull() ?: return null
        val acceptedNames = setOf(
            receiver.first(),
            receiver.joinToString(""),
            receiver.last()
        ).map { name -> name + EXT_SUFFIX }
        if (file.baseName in acceptedNames) return null
        val expectedName = receiver.first() + EXT_SUFFIX
        return "Rename ${file.baseName}.kt to $expectedName.kt: a util file of ${receiver.joinToString(".")} " +
            "extensions is named after its receiver."
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
                sources.createViolation(
                    RULE,
                    file,
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
