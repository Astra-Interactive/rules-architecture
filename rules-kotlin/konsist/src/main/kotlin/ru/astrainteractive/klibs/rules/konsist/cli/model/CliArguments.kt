package ru.astrainteractive.klibs.rules.konsist.cli.model

import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Vocabulary
import java.io.File

/**
 * The options of the command line, each written as `--name value`; `--main` and `--test` may repeat and accept
 * either a directory or `sourceSet=directory`; `--entry-point-word`, `--technology-word` and `--plural-concept`
 * may repeat and add the project's own words to the defaults; `--dependency` may repeat and names a main source
 * directory of a unit this one depends on, whose models the checks compare with this unit's types;
 * `--android-namespace` is the `namespace` of an Android unit.
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
    val mainSourceSets: Map<String, File>,
    val testSourceSets: Map<String, File>,
    val reportFile: File,
    val vocabulary: Vocabulary,
    val dependencyDirectories: List<File>,
    val androidNamespace: String?
) {
    companion object {
        const val PROJECT_DIR = "--project-dir"
        const val UNIT_KIND = "--unit-kind"
        const val ROOT_PACKAGE = "--root-package"
        const val UNIT_PATH = "--unit-path"
        const val MAIN = "--main"
        const val TEST = "--test"
        const val REPORT = "--report"
        const val ENTRY_POINT_WORD = "--entry-point-word"
        const val TECHNOLOGY_WORD = "--technology-word"
        const val PLURAL_CONCEPT = "--plural-concept"
        const val DEPENDENCY = "--dependency"
        const val ANDROID_NAMESPACE = "--android-namespace"

        private val OPTIONS = setOf(
            PROJECT_DIR,
            UNIT_KIND,
            ROOT_PACKAGE,
            UNIT_PATH,
            MAIN,
            TEST,
            REPORT,
            ENTRY_POINT_WORD,
            TECHNOLOGY_WORD,
            PLURAL_CONCEPT,
            DEPENDENCY,
            ANDROID_NAMESPACE
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
            val kinds = UnitKind.entries.joinToString(transform = UnitKind::cliName)
            return requireNotNull(UnitKind.fromCliName(name)) { "Unknown unit kind '$name', expected one of $kinds" }
        }

        private fun sourceSets(values: List<String>): Map<String, File> {
            return buildMap {
                values.filter { value -> "=" in value }.forEach { value ->
                    put(value.substringBefore('='), File(value.substringAfter('=')))
                }
            }
        }

        private fun sourceDirectory(value: String): File = File(value.substringAfter('='))

        fun parse(arguments: List<String>): CliArguments {
            require(arguments.size % 2 == 0) { "Every option takes exactly one value: $arguments" }
            val options = arguments.chunked(2).groupBy(
                keySelector = List<String>::first,
                valueTransform = List<String>::last
            )
            val unknown = options.keys - OPTIONS
            require(unknown.isEmpty()) { "Unknown options $unknown, expected some of $OPTIONS" }
            return CliArguments(
                projectDirectory = File(options.required(PROJECT_DIR)),
                unitKind = parseUnitKind(options.required(UNIT_KIND)),
                rootPackage = options.single(ROOT_PACKAGE),
                unitPath = options.single(UNIT_PATH),
                mainDirectories = options[MAIN].orEmpty().map(::sourceDirectory),
                testDirectories = options[TEST].orEmpty().map(::sourceDirectory),
                mainSourceSets = sourceSets(options[MAIN].orEmpty()),
                testSourceSets = sourceSets(options[TEST].orEmpty()),
                reportFile = File(options.required(REPORT)),
                vocabulary = Vocabulary(
                    entryPointWords = options[ENTRY_POINT_WORD].orEmpty().toSet(),
                    technologyWords = options[TECHNOLOGY_WORD].orEmpty().toSet(),
                    pluralConcepts = options[PLURAL_CONCEPT].orEmpty().toSet()
                ),
                dependencyDirectories = options[DEPENDENCY].orEmpty().map(::File),
                androidNamespace = options.single(ANDROID_NAMESPACE)
            )
        }
    }
}
