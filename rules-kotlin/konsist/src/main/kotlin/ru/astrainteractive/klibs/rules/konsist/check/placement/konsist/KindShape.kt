package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoBaseDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.provider.KoNameProvider
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.baseName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelNames
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Placement
import ru.astrainteractive.klibs.rules.konsist.layout.util.describe

/**
 * Checks the shape `<unit root>[/<concept>]/<kind>/<File>`: no file lies loose in the unit root or in a concept
 * package, and a kind package holds files, not packages. A technology unit's concept packages hold its
 * implementations, technology packages may hold their own kinds (`table`), and the entry point keeps its main
 * class at the root.
 *
 * Compose's `composable` is the one kind that may hold sub-packages, one level of them, each holding files only
 * (`screen`, `component`, `layout`), never `model`, `util` or `theme`, and never next to files of `composable`
 * itself. `PluralPackageName` judges their number.
 */
internal class KindShape : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun packagePath(placement: Placement): String {
        val segments = placement.concepts + listOfNotNull(placement.kindWord, placement.subKind) + placement.below
        return segments.joinToString("/")
    }

    private fun kindPath(placement: Placement): String {
        return placement.concepts.plus(listOfNotNull(placement.kindWord)).joinToString("/")
    }

    private fun findComposableMessage(file: KoFileDeclaration, placement: Placement, subKind: String): String? {
        val path = packagePath(placement)
        if (subKind in LEVEL_KINDS) {
            return "Move ${file.baseName} out of '$path': $subKind is a kind of the level, never a sub-package of " +
                "composable."
        }
        if (placement.below.isEmpty()) return null
        return "Move ${file.baseName} out of '$path': a composable sub-package holds only files."
    }

    private fun declaresOnlyInterfacesNamed(file: KoFileDeclaration, suffix: String): Boolean {
        val interfaceNames = file.interfaces(includeNested = false).map(KoNameProvider::name)
        val allEndWithSuffix = interfaceNames.all { name -> name.endsWith(suffix) }
        return interfaceNames.isNotEmpty() && allEndWithSuffix && file.topLevelNames().size == interfaceNames.size
    }

    private fun findAppApiMessage(sources: UnitSources, file: KoFileDeclaration, placement: Placement): String? {
        val isAppApi = sources.kind == UnitKind.API && sources.context.productKind == ProductKind.APP
        val kindWord = placement.kindWord ?: return null
        if (!isAppApi) return null
        val path = packagePath(placement)
        val stateHolderSuffix = STATE_HOLDER_SUFFIXES[kindWord]
        if (stateHolderSuffix != null) {
            if (declaresOnlyInterfacesNamed(file, stateHolderSuffix)) return null
            return "Move ${file.baseName} out of '$path': an app's api unit holds only interfaces named " +
                "...$stateHolderSuffix in $kindWord; move the implementation to the feature's impl unit."
        }
        if (kindWord in APP_API_KIND_WORDS) return null
        if (kindWord == MEMORY_WORD) {
            return "Move ${file.baseName} out of '$path': an app's api unit holds no memory implementation; move " +
                "it to the feature's impl unit, or make it a stub."
        }
        return "Move ${file.baseName} out of '$path': an app's api unit holds only the " +
            "${APP_API_KIND_WORDS.joinToString(", ")} kinds, not $kindWord."
    }

    private fun findMessage(sources: UnitSources, file: KoFileDeclaration, placement: Placement): String? {
        val kind = placement.kind
        if (kind == null && placement.concepts.isEmpty()) {
            if (sources.kind == UnitKind.ENTRY_POINT) return null
            return "Move ${file.baseName} into a kind package: the unit root holds only packages."
        }
        if (kind == null) {
            return "Move ${file.baseName} into a kind package of ${placement.describe()}: a concept package " +
                "holds only packages."
        }
        val appApiMessage = findAppApiMessage(
            sources,
            file,
            placement
        )
        if (appApiMessage != null) return appApiMessage
        val subKind = placement.subKind
        if (subKind != null) {
            return findComposableMessage(
                file,
                placement,
                subKind
            )
        }
        if (placement.below.isEmpty() || kind == Kind.IMPLEMENTATION) return null
        return "Move ${file.baseName} out of '${packagePath(placement)}': a kind package holds only files."
    }

    private fun findMixedComposables(sources: UnitSources): List<Violation> {
        val splitComposables = sources.mainFiles
            .mapNotNull(sources::placementOf)
            .filter { placement -> placement.subKind != null }
            .map(::kindPath)
            .toSet()
        return sources.mainFiles.mapNotNull { file ->
            val placement = sources.placementOf(file) ?: return@mapNotNull null
            val isComposableFile = placement.kindWord == Kind.COMPOSABLE_WORD && placement.subKind == null
            if (!isComposableFile || kindPath(placement) !in splitComposables) return@mapNotNull null
            val message = "Move ${file.baseName} into a sub-package of ${placement.describe()}: composable holds " +
                "files or sub-packages, never both."
            sources.createViolation(
                RULE,
                file,
                file.baseName,
                message
            )
        }
    }

    private fun <T> createNotClassViolations(
        sources: UnitSources,
        file: KoFileDeclaration,
        placement: Placement,
        declarations: List<T>
    ): List<Violation> where T : KoBaseDeclaration, T : KoNameProvider {
        return declarations.map { declaration ->
            sources.createViolation(
                RULE,
                file,
                declaration,
                declaration.name,
                "Move ${declaration.name} out of '${packagePath(placement)}': a role kind holds classes only."
            )
        }
    }

    private fun checkRoleKindFile(
        sources: UnitSources,
        file: KoFileDeclaration,
        placement: Placement
    ): List<Violation> {
        val suffix = checkNotNull(placement.kindWord).replaceFirstChar(Char::uppercase)
        val misnamed = file.topLevelClasses()
            .filterNot { declaration -> declaration.name.endsWith(suffix) }
            .map { declaration ->
                sources.createViolation(
                    RULE,
                    file,
                    declaration,
                    declaration.name,
                    "Rename ${declaration.name} so that its name ends with $suffix: a role kind holds the classes " +
                        "its word names."
                )
            }
        val functions = createNotClassViolations(
            sources,
            file,
            placement,
            file.functions(includeNested = false, includeLocal = false)
        )
        val types = createNotClassViolations(
            sources,
            file,
            placement,
            file.interfaces(includeNested = false) + file.objects(includeNested = false)
        )
        return misnamed + functions + types
    }

    private fun findRoleKindViolations(sources: UnitSources): List<Violation> {
        return sources.mainFiles.flatMap { file ->
            val placement = sources.placementOf(file)
            if (placement?.kind != Kind.ROLE) return@flatMap emptyList()
            checkRoleKindFile(
                sources,
                file,
                placement
            )
        }
    }

    override fun check(sources: UnitSources): List<Violation> {
        val shapeViolations = sources.mainFiles.mapNotNull { file ->
            val placement = sources.placementOf(file) ?: return@mapNotNull null
            val message = findMessage(
                sources,
                file,
                placement
            ) ?: return@mapNotNull null
            sources.createViolation(
                RULE,
                file,
                file.baseName,
                message
            )
        }
        return shapeViolations + findMixedComposables(sources) + findRoleKindViolations(sources)
    }

    private companion object {
        const val RULE = "KindShape"
        const val MEMORY_WORD = "memory"

        @JvmField
        val STATE_HOLDER_SUFFIXES = mapOf(
            "component" to "Component",
            "viewmodel" to "ViewModel"
        )

        @JvmField
        val APP_API_KIND_WORDS = listOf(
            "api",
            "model",
            "util",
            "usecase",
            "stub",
            "di",
            "component",
            "viewmodel"
        )

        @JvmField
        val LEVEL_KINDS = setOf(
            "model",
            "util",
            "theme"
        )
    }
}
