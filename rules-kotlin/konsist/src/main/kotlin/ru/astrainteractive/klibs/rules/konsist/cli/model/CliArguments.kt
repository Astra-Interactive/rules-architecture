package ru.astrainteractive.klibs.rules.konsist.cli.model

import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.util.toUnitContext
import ru.astrainteractive.klibs.rules.core.unit.util.toUnitKindOrNull
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Vocabulary
import java.io.File

/**
 * The options of the command line, each written as `--name value`; `--main` and `--test` may repeat and accept
 * either a directory or `sourceSet=directory`; `--entry-point-word`, `--technology-word`, `--plural-concept`,
 * `--platform-class`, `--extension-kind` and `--role-kind` may repeat and add the project's own words to the defaults;
 * `--dependency` may repeat and names a main source directory of a unit this one depends on, whose models the checks
 * compare with this unit's types; `--android-namespace` is the `namespace` of an Android unit.
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
    val androidNamespace: String?,
    val unitContext: UnitContext?
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
        const val PLATFORM_CLASS = "--platform-class"
        const val EXTENSION_KIND = "--extension-kind"
        const val ROLE_KIND = "--role-kind"
        const val DEPENDENCY = "--dependency"
        const val ANDROID_NAMESPACE = "--android-namespace"
        const val UNIT_CONTEXT = "--unit-context"

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
            PLATFORM_CLASS,
            EXTENSION_KIND,
            ROLE_KIND,
            DEPENDENCY,
            ANDROID_NAMESPACE,
            UNIT_CONTEXT
        )

        /**
         * `database` is a fixed kind of `arch-context.md` that [Kind] leaves to the placement of a technology unit.
         */
        private val FIXED_KIND_WORDS = Kind.entries.flatMap(Kind::words).toSet() + "database"

        /**
         * The last words of `arch-package.md` that rename a class instead of naming its kind.
         */
        private val RENAMED_WORDS = setOf(
            "controller",
            "helper",
            "manager",
            "utils"
        )

        /**
         * The last words whose classes a placement check moves to a fixed kind wherever they sit: `…Module` to `di`
         * (`DiPlacement`), so a role kind of that word could hold none of them.
         */
        private val PLACED_WORDS = mapOf("module" to "di")

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
            return requireNotNull(name.toUnitKindOrNull()) { "Unknown unit kind '$name', expected one of $kinds" }
        }

        private fun sourceSets(values: List<String>): Map<String, File> {
            return buildMap {
                values.filter { value -> "=" in value }.forEach { value ->
                    put(value.substringBefore('='), File(value.substringAfter('=')))
                }
            }
        }

        private fun sourceDirectory(value: String): File = File(value.substringAfter('='))

        private fun checkRoleKinds(vocabulary: Vocabulary, unitContext: UnitContext?) {
            val takenWords = FIXED_KIND_WORDS + vocabulary.entryPointWords + vocabulary.extensionKinds +
                vocabulary.technologyWords + unitContext?.technologyWords.orEmpty()
            vocabulary.roleKinds.forEach { word ->
                require(word == word.trim().lowercase()) {
                    "$ROLE_KIND '$word' must be lower-case and trimmed: a role kind is a package segment, the last " +
                        "word of its classes in lower case."
                }
                require(word !in takenWords) {
                    "$ROLE_KIND '$word' is a fixed kind, a technology or an entry-point word of the project; a role " +
                        "kind is named by a word of its own."
                }
                val className = "...${word.replaceFirstChar(Char::uppercase)}"
                require(word !in RENAMED_WORDS) {
                    "$ROLE_KIND '$word' names no role: a class named $className is renamed, not given a kind."
                }
                require(word !in Kind.FORBIDDEN_WORDS) {
                    "$ROLE_KIND '$word' is a forbidden package name: a package is named for a concept or a kind, " +
                        "never a layer or a filler word."
                }
                val placedKind = PLACED_WORDS[word]
                require(placedKind == null) {
                    "$ROLE_KIND '$word' names classes another check places: a class named $className belongs to " +
                        "$placedKind."
                }
            }
        }

        fun parse(arguments: List<String>): CliArguments {
            require(arguments.size % 2 == 0) { "Every option takes exactly one value: $arguments" }
            val options = arguments.chunked(2).groupBy(
                keySelector = List<String>::first,
                valueTransform = List<String>::last
            )
            val unknown = options.keys - OPTIONS
            require(unknown.isEmpty()) { "Unknown options $unknown, expected some of $OPTIONS" }
            val vocabulary = Vocabulary(
                entryPointWords = options[ENTRY_POINT_WORD].orEmpty().toSet(),
                technologyWords = options[TECHNOLOGY_WORD].orEmpty().toSet(),
                pluralConcepts = options[PLURAL_CONCEPT].orEmpty().toSet(),
                platformClasses = options[PLATFORM_CLASS].orEmpty().toSet(),
                extensionKinds = options[EXTENSION_KIND].orEmpty().toSet(),
                roleKinds = options[ROLE_KIND].orEmpty().toSet()
            )
            val unitContext = options.single(UNIT_CONTEXT)?.toUnitContext()
            checkRoleKinds(vocabulary, unitContext)
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
                vocabulary = vocabulary,
                dependencyDirectories = options[DEPENDENCY].orEmpty().map(::File),
                androidNamespace = options.single(ANDROID_NAMESPACE),
                unitContext = unitContext
            )
        }
    }
}
