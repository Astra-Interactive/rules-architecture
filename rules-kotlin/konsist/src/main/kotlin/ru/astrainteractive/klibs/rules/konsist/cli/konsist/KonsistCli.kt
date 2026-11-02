@file:JvmName("KonsistCli")

package ru.astrainteractive.klibs.rules.konsist.cli.konsist

import ru.astrainteractive.klibs.rules.core.unit.model.RuleId
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.AnonymousProjectObject
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.DiModuleVisibility
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.EntryPointVisibility
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.ImplementationVisibility
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.NestedDataClass
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.NestedEnum
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.SealedFamilyFile
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.ThemeVisibility
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.UseCaseAbstraction
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.UseCaseShape
import ru.astrainteractive.klibs.rules.konsist.check.direction.konsist.KindImportDirection
import ru.astrainteractive.klibs.rules.konsist.check.direction.konsist.PackageDependencyCycle
import ru.astrainteractive.klibs.rules.konsist.check.direction.konsist.PlatformImport
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureChecker
import ru.astrainteractive.klibs.rules.konsist.check.konsist.SuppressionPolicy
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.AndroidNamespace
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.ComposeKindWithoutCompose
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.ContractPlacement
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.ContractUnitShape
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.CoreShape
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.DiPlacement
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.EntryPointShape
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.ErrorPlacement
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.FakePlacement
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.ForbiddenPackageName
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.InMemoryPlacement
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.KindShape
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.PlatformPackage
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.PluralPackageName
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.RootModulePlacement
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.RootPackagePath
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.TestMirror
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.UtilFileName
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.UtilFunctionAsExtension
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.UtilShape
import ru.astrainteractive.klibs.rules.konsist.cli.model.CliArguments

internal fun registeredChecks(): Map<RuleId, ArchitectureCheck> {
    return mapOf(
        RuleId.ANDROID_NAMESPACE to AndroidNamespace(),
        RuleId.ANONYMOUS_PROJECT_OBJECT to AnonymousProjectObject(),
        RuleId.COMPOSE_KIND_WITHOUT_COMPOSE to ComposeKindWithoutCompose(),
        RuleId.CONTRACT_PLACEMENT to ContractPlacement(),
        RuleId.CONTRACT_UNIT_SHAPE to ContractUnitShape(),
        RuleId.CORE_SHAPE to CoreShape(),
        RuleId.DI_MODULE_VISIBILITY to DiModuleVisibility(),
        RuleId.DI_PLACEMENT to DiPlacement(),
        RuleId.ENTRY_POINT_SHAPE to EntryPointShape(),
        RuleId.ENTRY_POINT_VISIBILITY to EntryPointVisibility(),
        RuleId.ERROR_PLACEMENT to ErrorPlacement(),
        RuleId.FAKE_PLACEMENT to FakePlacement(),
        RuleId.FORBIDDEN_PACKAGE_NAME to ForbiddenPackageName(),
        RuleId.IMPLEMENTATION_VISIBILITY to ImplementationVisibility(),
        RuleId.IN_MEMORY_PLACEMENT to InMemoryPlacement(),
        RuleId.KIND_IMPORT_DIRECTION to KindImportDirection(),
        RuleId.KIND_SHAPE to KindShape(),
        RuleId.NESTED_DATA_CLASS to NestedDataClass(),
        RuleId.NESTED_ENUM to NestedEnum(),
        RuleId.PACKAGE_DEPENDENCY_CYCLE to PackageDependencyCycle(),
        RuleId.PLATFORM_IMPORT to PlatformImport(),
        RuleId.PLATFORM_PACKAGE to PlatformPackage(),
        RuleId.PLURAL_PACKAGE_NAME to PluralPackageName(),
        RuleId.ROOT_MODULE_PLACEMENT to RootModulePlacement(),
        RuleId.ROOT_PACKAGE_PATH to RootPackagePath(),
        RuleId.SEALED_FAMILY_FILE to SealedFamilyFile(),
        RuleId.TEST_MIRROR to TestMirror(),
        RuleId.THEME_VISIBILITY to ThemeVisibility(),
        RuleId.USE_CASE_ABSTRACTION to UseCaseAbstraction(),
        RuleId.USE_CASE_SHAPE to UseCaseShape(),
        RuleId.UTIL_FILE_NAME to UtilFileName(),
        RuleId.UTIL_FUNCTION_AS_EXTENSION to UtilFunctionAsExtension(),
        RuleId.UTIL_SHAPE to UtilShape(),
    )
}

/**
 * Checks the sources of one build unit against the architecture rules and writes the violations to the report
 * file. Finishes normally whether violations were found or not: the caller decides what fails the build.
 *
 * Options: `--project-dir`, `--unit-kind` (`context`, `library`, `api`, `contract`, `entry-point`,
 * `technology`), `--report`, an optional `--unit-path` or `--root-package`, and any number of `--main` and
 * `--test` source directories, written as `sourceSet=path` when source-set mirroring should be checked, and any
 * number of `--entry-point-word`, `--technology-word` and `--plural-concept` for the project's own words and of
 * `--dependency` for the main source directories of the units this one depends on, and an optional
 * `--android-namespace`.
 */
public fun main(args: Array<String>) {
    val arguments = CliArguments.parse(args.toList())
    val sources = UnitSources.load(
        kind = arguments.unitKind,
        rootPackage = arguments.rootPackage,
        unitPath = arguments.unitPath,
        mainDirectories = arguments.mainDirectories,
        testDirectories = arguments.testDirectories,
        mainSourceSets = arguments.mainSourceSets,
        testSourceSets = arguments.testSourceSets,
        projectDirectory = arguments.projectDirectory,
        vocabulary = arguments.vocabulary,
        dependencyDirectories = arguments.dependencyDirectories,
        androidNamespace = arguments.androidNamespace,
        context = arguments.unitContext
    )
    val checker = ArchitectureChecker(registeredChecks(), SuppressionPolicy())
    writeReport(checker.check(sources), arguments.reportFile)
}
