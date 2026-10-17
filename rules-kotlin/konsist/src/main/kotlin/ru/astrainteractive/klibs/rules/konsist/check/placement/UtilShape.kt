package ru.astrainteractive.klibs.rules.konsist.check.placement

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.simpleName
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelTypeNames
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Checks `util` both ways: a `util` package holds functions, never a class, an interface or an object; and an
 * extension function lives in `util/<Subject>Ext.kt`, unless it sits in the file of its receiver type or of a
 * type argument of it (`Collection<Coloring>` next to `Coloring`), as part of that type. Entry-point kinds are
 * not judged: a project may write its entry points as extensions (AstraRating's `slot` builders).
 */
internal class UtilShape : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun typesInUtil(sources: UnitSources, file: KoFileDeclaration): List<Violation> {
        return file.topLevelTypeNames().map { name ->
            val message = "Move $name out of util: util holds functions only, a class goes to the kind its role names."
            Violation(RULE, sources.relativePath(file), name, message)
        }
    }

    private fun extensionsOutsideUtil(sources: UnitSources, file: KoFileDeclaration): List<Violation> {
        val ownTypes = file.topLevelTypeNames().toSet()
        return file.functions(includeNested = false, includeLocal = false).mapNotNull { function ->
            val receiverType = function.receiverType ?: return@mapNotNull null
            val receiver = receiverType.simpleName
            val receiverTypeNames = TYPE_NAME.findAll(receiverType.text).map { match -> match.value }.toSet()
            if (receiverTypeNames.any { typeName -> typeName in ownTypes }) return@mapNotNull null
            val message = "Move the extension $receiver.${function.name} to util/${receiver}Ext.kt of the level " +
                "that uses it."
            Violation(RULE, sources.relativePath(file), "$receiver.${function.name}", message)
        }
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles.flatMap { file ->
            val placement = sources.placementOf(file) ?: return@flatMap emptyList()
            when (placement.kind) {
                Kind.UTIL -> typesInUtil(sources, file)
                Kind.ENTRY_POINT -> emptyList()
                else -> extensionsOutsideUtil(sources, file)
            }
        }
    }

    private companion object {
        const val RULE = "UtilShape"

        @JvmField
        val TYPE_NAME = Regex("""\b[A-Z]\w*""")
    }
}
