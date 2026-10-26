@file:JvmName("KonsistCli")

package ru.astrainteractive.klibs.rules.konsist.cli.konsist

import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.ContractName
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.DuplicateRepresentation
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.EntryPointVisibility
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.FieldDependencyConstruction
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.ImplementationVisibility
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.NestedDataClass
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.NestedEnum
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.SealedFamilyFile
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.StateTypePlacement
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.UseCaseAbstraction
import ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist.UseCaseShape
import ru.astrainteractive.klibs.rules.konsist.check.direction.konsist.KindImportDirection
import ru.astrainteractive.klibs.rules.konsist.check.direction.konsist.PlatformImport
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureChecker
import ru.astrainteractive.klibs.rules.konsist.check.konsist.SuppressionPolicy
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
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
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.RootModulePlacement
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.TestMirror
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.TestOwnerBehaviour
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.UtilFileName
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.UtilFunctionAsExtension
import ru.astrainteractive.klibs.rules.konsist.check.placement.konsist.UtilShape
import ru.astrainteractive.klibs.rules.konsist.cli.model.CliArguments

/**
 * Checks the sources of one build unit against the architecture rules and writes the violations to the report
 * file. Finishes normally whether violations were found or not: the caller decides what fails the build.
 *
 * Options: `--project-dir`, `--unit-kind` (`context`, `library`, `api`, `contract`, `entry-point`,
 * `technology`), `--report`, an optional `--unit-path` or `--root-package`, and any number of `--main` and
 * `--test` source directories, written as `sourceSet=path` when source-set mirroring should be checked.
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
        projectDirectory = arguments.projectDirectory
    )
    val checker = ArchitectureChecker(
        listOf(
            ComposeKindWithoutCompose(),
            ContractName(),
            ContractPlacement(),
            ContractUnitShape(),
            CoreShape(),
            DiPlacement(),
            DuplicateRepresentation(),
            EntryPointShape(),
            EntryPointVisibility(),
            ErrorPlacement(),
            FakePlacement(),
            FieldDependencyConstruction(),
            ForbiddenPackageName(),
            ImplementationVisibility(),
            InMemoryPlacement(),
            KindImportDirection(),
            KindShape(),
            NestedDataClass(),
            NestedEnum(),
            PlatformImport(),
            PlatformPackage(),
            RootModulePlacement(),
            SealedFamilyFile(),
            StateTypePlacement(),
            TestMirror(),
            TestOwnerBehaviour(),
            UseCaseAbstraction(),
            UseCaseShape(),
            UtilFileName(),
            UtilFunctionAsExtension(),
            UtilShape(),
        ),
        SuppressionPolicy()
    )
    writeReport(checker.check(sources), arguments.reportFile)
}
