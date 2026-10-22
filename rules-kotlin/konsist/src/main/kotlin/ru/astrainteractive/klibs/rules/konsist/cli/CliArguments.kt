package ru.astrainteractive.klibs.rules.konsist.cli

import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File

/**
 * The options of the command line, each written as `--name value`; `--main` and `--test` may repeat.
 *
 * @property unitPath the Gradle path of the unit (`:modules:event-bukkit`), which finds the root package when
 * [rootPackage] is not given
 */
internal data class CliArguments(
    val projectDirectory: File,
    val unitKind: UnitKind,
    val rootPackage: String?,
    val unitPath: String?,
    val mainDirectories: List<File>,
    val testDirectories: List<File>,
    val reportFile: File
) {
    companion object {
        const val PROJECT_DIR = "--project-dir"
        const val UNIT_KIND = "--unit-kind"
        const val ROOT_PACKAGE = "--root-package"
        const val UNIT_PATH = "--unit-path"
        const val MAIN = "--main"
        const val TEST = "--test"
        const val REPORT = "--report"

        private val OPTIONS = setOf(
            PROJECT_DIR,
            UNIT_KIND,
            ROOT_PACKAGE,
            UNIT_PATH,
            MAIN,
            TEST,
            REPORT
        )

        private fun Map<String, List<String>>.single(name: String): String? {
            val values = get(name) ?: return null
            require(values.size == 1) { "$name is given ${values.size} times, expected once" }
            return values.single()
        }

        private fun Map<String, List<String>>.required(name: String): String {
            return requireNotNull(single(name)) { "$name is required" }
        }

        private fun parseUnitKind(name: String): UnitKind {
            return requireNotNull(UnitKind.fromCliName(name)) {
                "Unknown unit kind '$name', expected one of ${UnitKind.entries.joinToString { kind -> kind.cliName }}"
            }
        }

        fun parse(arguments: List<String>): CliArguments {
            require(arguments.size % 2 == 0) { "Every option takes exactly one value: $arguments" }
            val options = arguments.chunked(2).groupBy(
                keySelector = { option -> option.first() },
                valueTransform = { option -> option.last() }
            )
            val unknown = options.keys - OPTIONS
            require(unknown.isEmpty()) { "Unknown options $unknown, expected some of $OPTIONS" }
            return CliArguments(
                projectDirectory = File(options.required(PROJECT_DIR)),
                unitKind = parseUnitKind(options.required(UNIT_KIND)),
                rootPackage = options.single(ROOT_PACKAGE),
                unitPath = options.single(UNIT_PATH),
                mainDirectories = options[MAIN].orEmpty().map(::File),
                testDirectories = options[TEST].orEmpty().map(::File),
                reportFile = File(options.required(REPORT))
            )
        }
    }
}
