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
 * extension function lives in `util/<Subject>Ext.kt`, never in the file of its receiver type. The subject is the
 * receiver type, or its first type argument when the receiver is a standard collection, flow or function type
 * (`Collection<Coloring>` goes to `ColoringExt.kt`). Entry-point kinds are not judged: a project may write its
 * entry points as extensions (AstraRating's `slot` builders).
 */
internal class UtilShape : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun typesInUtil(sources: UnitSources, file: KoFileDeclaration): List<Violation> {
        return file.topLevelTypeNames().map { name ->
            val message = "Move $name out of util: util holds functions only, a class goes to the kind its role names."
            Violation(RULE, sources.relativePath(file), name, message)
        }
    }

    private fun findSubject(receiverText: String, receiver: String): String {
        val typeNames = TYPE_NAME.findAll(receiverText).map { match -> match.value }.toList()
        if (receiverText.contains(FUNCTION_ARROW)) return typeNames.firstOrNull() ?: receiver
        if (receiver !in CONTAINER_TYPES) return receiver
        return typeNames.drop(1).firstOrNull() ?: receiver
    }

    private fun extensionsOutsideUtil(sources: UnitSources, file: KoFileDeclaration): List<Violation> {
        return file.functions(includeNested = false, includeLocal = false).mapNotNull { function ->
            val receiverType = function.receiverType ?: return@mapNotNull null
            val receiver = receiverType.simpleName
            val subject = findSubject(receiverType.text, receiver)
            val message = "Move the extension $receiver.${function.name} to util/${subject}Ext.kt of the level " +
                "that uses it: the file is named after its subject, the element type of a collection, flow or " +
                "function type."
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
        const val FUNCTION_ARROW = "->"

        @JvmField
        val TYPE_NAME = Regex("""\b[A-Z]\w*""")

        @JvmField
        val CONTAINER_TYPES = setOf(
            "Array",
            "Collection",
            "Flow",
            "Iterable",
            "List",
            "Map",
            "MutableCollection",
            "MutableIterable",
            "MutableList",
            "MutableMap",
            "MutableSet",
            "MutableSharedFlow",
            "MutableStateFlow",
            "Sequence",
            "Set",
            "SharedFlow",
            "StateFlow",
        )
    }
}
