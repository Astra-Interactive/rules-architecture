package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.declaration.KoImportDeclaration
import com.lemonappdev.konsist.api.declaration.KoInterfaceDeclaration
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.nestedDataTypeNames
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.signatureTypeTexts
import ru.astrainteractive.klibs.rules.konsist.check.model.util.simpleName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelTypeNames
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind

/**
 * Checks `util` both ways: a `util` package holds functions, never a class, an interface or an object, and none of
 * them names a contract of the project as its receiver, a parameter, a context parameter or a type-parameter bound
 * (`Repo?`, `List<Repo>` and a type alias of one included; a data type nested in a contract is no contract); and an
 * extension function lives in `util/<Subject>Ext.kt`, never in the file of its receiver type. The subject is the
 * receiver type, or its first type argument when the receiver is a standard collection, flow or function type
 * (`Collection<Coloring>` goes to `ColoringExt.kt`). Two exceptions: a `private` extension in a Compose composable
 * file, which only that file uses, and any extension in a UI kind the project lists in `extensionKinds` (`slot`, where
 * `fun SlotContext.backSlot()` in `BackSlot.kt` is the slot itself).
 */
internal class UtilShape : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun typesInUtil(sources: UnitSources, file: KoFileDeclaration): List<Violation> {
        val declarations = file.topLevelClasses() + file.interfaces(includeNested = false) +
            file.objects(includeNested = false)
        return declarations.map { declaration ->
            val message = "Move ${declaration.name} out of util: util holds functions only, a class goes to the " +
                "kind its role names."
            sources.createViolation(
                RULE,
                file,
                declaration,
                declaration.name,
                message
            )
        }
    }

    private fun resolveCandidates(file: KoFileDeclaration, reference: String): List<String> {
        val head = reference.substringBefore('.')
        val nested = reference.removePrefix(head)
        if (head.first().isLowerCase()) return listOf(reference)
        val explicit = file.imports
            .filter { declaration ->
                val visibleName = declaration.alias?.name ?: declaration.name.substringAfterLast('.')
                !declaration.isWildcard && visibleName == head
            }
            .map(KoImportDeclaration::name)
        val wildcard = file.imports.filter(KoImportDeclaration::isWildcard)
            .map { declaration -> "${declaration.name}.$head" }
        val candidates = explicit.ifEmpty { wildcard.plus("${file.packageName}.$head") }
        return candidates.map { name -> name + nested }
    }

    private fun findContractType(file: KoFileDeclaration, typeText: String, contracts: Set<String>): String? {
        return TYPE_REFERENCE.findAll(typeText)
            .map(MatchResult::value)
            .flatMap { reference -> resolveCandidates(file, reference) }
            .firstOrNull { name -> name in contracts }
    }

    private fun findProjectContracts(sources: UnitSources): Set<String> {
        val mainContracts = sources.mainFiles.filter { file -> sources.placementOf(file)?.kind == Kind.CONTRACT }
        val dependencyContracts = sources.dependencyFiles.filter { file ->
            Kind.fromSegment(file.packageName.substringAfterLast('.'), sources.layout.vocabulary) == Kind.CONTRACT
        }
        val interfaces = mainContracts.plus(dependencyContracts)
            .flatMap { file -> file.interfaces(includeNested = true) }
        val dataTypes = interfaces.flatMap(KoInterfaceDeclaration::nestedDataTypeNames).toSet()
        val contracts = interfaces.mapNotNull(KoInterfaceDeclaration::fullyQualifiedName)
            .filterNot { name -> dataTypes.any { dataType -> name == dataType || name.startsWith("$dataType.") } }
            .toSet()
        val aliases = sources.mainFiles.plus(sources.dependencyFiles).flatMap { file ->
            file.typeAliases
                .filter { alias ->
                    val contract = findContractType(
                        file,
                        alias.type.text,
                        contracts
                    )
                    contract != null
                }
                .map { alias -> "${file.packageName}.${alias.name}".trimStart('.') }
        }
        return contracts + aliases
    }

    private fun checkContractParameters(
        sources: UnitSources,
        file: KoFileDeclaration,
        contracts: Set<String>
    ): List<Violation> {
        return file.functions(includeNested = false, includeLocal = false).mapNotNull { function ->
            val receiverType = function.receiverType
            val contract = function.signatureTypeTexts().firstNotNullOfOrNull { typeText ->
                findContractType(
                    file,
                    typeText,
                    contracts
                )
            } ?: return@mapNotNull null
            val subject = if (receiverType == null) function.name else "${receiverType.simpleName}.${function.name}"
            val message = "Move ${function.name} out of util: a util function takes no contract " +
                "(${contract.substringAfterLast('.')}); a flow over a contract is a use case, a single call is made " +
                "by the caller."
            sources.createViolation(
                RULE,
                file,
                function,
                subject,
                message
            )
        }
    }

    private fun findSubject(receiverText: String, receiver: String): String {
        val typeNames = TYPE_NAME.findAll(receiverText).map(MatchResult::value).toList()
        if (receiverText.contains(FUNCTION_ARROW)) return typeNames.firstOrNull() ?: receiver
        if (receiver !in CONTAINER_TYPES) return receiver
        return typeNames.drop(1).firstOrNull() ?: receiver
    }

    private fun isComposableFile(file: KoFileDeclaration): Boolean {
        return file.functions(includeNested = false, includeLocal = false)
            .any { function -> function.hasAnnotationWithName(COMPOSABLE) }
    }

    private fun isProjectReceiver(file: KoFileDeclaration, receiver: String, projectTypes: Set<String>): Boolean {
        if (receiver in projectTypes) return true
        val explicitImport = file.imports.singleOrNull { declaration ->
            val alias = declaration.alias
            val visibleName = if (alias == null) declaration.name.substringAfterLast('.') else alias.name
            !declaration.isWildcard && visibleName == receiver
        }
        if (explicitImport != null) return explicitImport.name in projectTypes
        if ("${file.packageName}.$receiver" in projectTypes) return true
        return file.imports.filter { declaration -> declaration.isWildcard }
            .any { declaration -> "${declaration.name}.$receiver" in projectTypes }
    }

    private fun extensionsOutsideUtil(
        sources: UnitSources,
        file: KoFileDeclaration,
        projectTypes: Set<String>
    ): List<Violation> {
        val allowsPrivateExtensions = isComposableFile(file)
        return file.functions(includeNested = false, includeLocal = false).mapNotNull { function ->
            val receiverType = function.receiverType ?: return@mapNotNull null
            val declaredReceiver = receiverType.text.substringBefore('<').removeSuffix("?")
            val projectReceiver = isProjectReceiver(
                file,
                declaredReceiver,
                projectTypes
            )
            if (allowsPrivateExtensions && function.hasPrivateModifier && !projectReceiver) return@mapNotNull null
            val receiver = receiverType.simpleName
            val subject = findSubject(receiverType.text, receiver)
            val message = "Move the extension $receiver.${function.name} to util/${subject}Ext.kt of the level " +
                "that uses it: the file is named after its subject, the element type of a collection, flow or " +
                "function type."
            sources.createViolation(
                RULE,
                file,
                function,
                "$receiver.${function.name}",
                message
            )
        }
    }

    override fun check(sources: UnitSources): List<Violation> {
        val projectTypes = sources.mainFiles.plus(sources.dependencyFiles).flatMap { file ->
            file.topLevelTypeNames().map { name -> "${file.packageName}.$name".trimStart('.') }
        }.toSet()
        val contracts = findProjectContracts(sources)
        return sources.mainFiles.flatMap { file ->
            val placement = sources.placementOf(file) ?: return@flatMap emptyList()
            when (placement.kind) {
                Kind.UTIL -> typesInUtil(sources, file) + checkContractParameters(
                    sources,
                    file,
                    contracts
                )
                else -> if (placement.kindWord in sources.layout.vocabulary.extensionKinds) {
                    emptyList()
                } else {
                    extensionsOutsideUtil(
                        sources,
                        file,
                        projectTypes
                    )
                }
            }
        }
    }

    private companion object {
        const val RULE = "UtilShape"
        const val FUNCTION_ARROW = "->"
        const val COMPOSABLE = "Composable"

        @JvmField
        val TYPE_NAME = Regex("""\b[A-Z]\w*""")

        @JvmField
        val TYPE_REFERENCE = Regex("""\b[A-Za-z_]\w*(?:\.[A-Za-z_]\w*)*""")

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
