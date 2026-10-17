@file:JvmName("KonsistCli")

package ru.astrainteractive.klibs.rules.konsist.cli

import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureChecker
import ru.astrainteractive.klibs.rules.konsist.check.declaration.EntryPointVisibility
import ru.astrainteractive.klibs.rules.konsist.check.declaration.NestedDataClass
import ru.astrainteractive.klibs.rules.konsist.check.declaration.UseCaseShape
import ru.astrainteractive.klibs.rules.konsist.check.direction.KindImportDirection
import ru.astrainteractive.klibs.rules.konsist.check.direction.PlatformImport
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.placement.ContractPlacement
import ru.astrainteractive.klibs.rules.konsist.check.placement.DiPlacement
import ru.astrainteractive.klibs.rules.konsist.check.placement.ErrorPlacement
import ru.astrainteractive.klibs.rules.konsist.check.placement.FakePlacement
import ru.astrainteractive.klibs.rules.konsist.check.placement.ForbiddenPackageName
import ru.astrainteractive.klibs.rules.konsist.check.placement.InMemoryPlacement
import ru.astrainteractive.klibs.rules.konsist.check.placement.KindShape
import ru.astrainteractive.klibs.rules.konsist.check.placement.TestMirror
import ru.astrainteractive.klibs.rules.konsist.check.placement.UtilShape

/**
 * Checks the sources of one build unit against the architecture rules and writes the violations to the report
 * file. Finishes normally whether violations were found or not: the caller decides what fails the build.
 *
 * Options: `--project-dir`, `--unit-kind` (`context`, `library`, `api`, `contract`, `entry-point`,
 * `technology`), `--report`, an optional `--unit-path` or `--root-package`, and any number of `--main` and
 * `--test` source directories.
 */
public fun main(args: Array<String>) {
    val arguments = CliArguments.parse(args.toList())
    val sources = UnitSources.load(
        kind = arguments.unitKind,
        rootPackage = arguments.rootPackage,
        unitPath = arguments.unitPath,
        mainDirectories = arguments.mainDirectories,
        testDirectories = arguments.testDirectories,
        projectDirectory = arguments.projectDirectory
    )
    val checker = ArchitectureChecker(
        listOf(
            ContractPlacement(),
            DiPlacement(),
            EntryPointVisibility(),
            ErrorPlacement(),
            FakePlacement(),
            ForbiddenPackageName(),
            InMemoryPlacement(),
            KindImportDirection(),
            KindShape(),
            NestedDataClass(),
            PlatformImport(),
            TestMirror(),
            UseCaseShape(),
            UtilShape(),
        )
    )
    writeReport(checker.check(sources), arguments.reportFile)
}
