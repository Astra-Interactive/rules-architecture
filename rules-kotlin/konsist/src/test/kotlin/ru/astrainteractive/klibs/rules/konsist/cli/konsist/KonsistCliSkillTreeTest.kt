@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.cli.konsist

import org.junit.jupiter.api.Assumptions.assumeTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.util.encode
import ru.astrainteractive.klibs.rules.konsist.check.model.SkillTreeReader
import ru.astrainteractive.klibs.rules.konsist.check.model.SkillTreeWriter
import ru.astrainteractive.klibs.rules.konsist.check.model.SkillUnit
import ru.astrainteractive.klibs.rules.konsist.layout.model.Vocabulary
import java.io.File
import java.security.MessageDigest
import kotlin.test.assertEquals

/**
 * Runs every registered check on each unit of the Kotlin worked trees of the package-layout skill
 * (`skills/package-layout/SKILL.md`, section 4), copied below with the SHA-256 of each fenced block.
 */
internal class KonsistCliSkillTreeTest {

    private class TreeCase(
        val units: List<SkillUnit>,
        val dependencies: Map<String, List<String>>,
        val entryPointWords: Set<String>,
        val productKind: ProductKind,
        val composeUnits: Set<String>
    )

    /**
     * The SHA-256 of a fenced block's lines joined by line feeds, without the fences.
     */
    private fun hashBlock(block: String): String {
        val digest = MessageDigest.getInstance("SHA-256").digest(block.toByteArray(Charsets.UTF_8))
        return digest.joinToString("") { byte -> "%02x".format(byte) }
    }

    /**
     * The fenced blocks of section 4 in the order they appear, except those under the Rust heading.
     */
    private fun readKotlinBlocks(skill: String): List<String> {
        val section = skill.substringAfter("\n## 4. ").substringBefore("\n## 5. ")
        val kotlin = section.substringBefore("\n### A Rust workspace")
        return kotlin.split("\n```")
            .filterIndexed { index, _ -> index % 2 == 1 }
            .map { block -> block.substringAfter('\n') }
    }

    private fun vocabularyOf(case: TreeCase): Vocabulary {
        return Vocabulary(
            entryPointWords = case.entryPointWords,
            technologyWords = emptySet(),
            pluralConcepts = emptySet(),
            platformClasses = emptySet(),
            extensionKinds = emptySet(),
            roleKinds = emptySet()
        )
    }

    private fun contextOf(case: TreeCase, unit: SkillUnit, mainDirectory: File): UnitContext {
        val compose = unit.path in case.composeUnits
        val sourceSets = mapOf("main" to setOf(mainDirectory.absolutePath))
        return UnitContext(
            unit.kind,
            case.productKind,
            false,
            emptySet(),
            listOf(
                UnitContext.Compilation(
                    "jvm:jvm:main",
                    sourceSets,
                    false,
                    compose,
                    case.dependencies.getValue(unit.path).map { path -> ":" + path.replace('/', ':') }.toSet()
                )
            )
        )
    }

    private fun checkUnit(case: TreeCase, unit: SkillUnit, directory: File): List<String> {
        val unitDirectory = File(directory, unit.path)
        val dependencies = case.dependencies.getValue(unit.path).map { path ->
            case.units.single { candidate -> candidate.path == path }
        }
        val sourceDirectories = SkillTreeWriter(case.units, vocabularyOf(case)).write(
            unit,
            dependencies,
            unitDirectory
        )
        val mainDirectory = sourceDirectories.first()
        val context = contextOf(
            case,
            unit,
            mainDirectory
        )
        val reportFile = File(unitDirectory, "build/architecture/konsist.tsv")
        val arguments = buildList {
            addAll(listOf("--project-dir", unitDirectory.absolutePath))
            addAll(listOf("--unit-kind", unit.kind.cliName))
            addAll(listOf("--unit-path", unit.gradlePath))
            addAll(listOf("--unit-context", context.encode()))
            addAll(listOf("--main", "main=${mainDirectory.absolutePath}"))
            addAll(listOf("--report", reportFile.absolutePath))
            case.entryPointWords.sorted().forEach { word -> addAll(listOf("--entry-point-word", word)) }
            sourceDirectories.drop(1).forEach { dependency -> addAll(listOf("--dependency", dependency.absolutePath)) }
        }

        main(arguments.toTypedArray())

        return reportFile.readLines()
    }

    private fun checkTree(case: TreeCase, directory: File): Map<String, List<String>> {
        return case.units.associate { unit ->
            val findings = checkUnit(
                case,
                unit,
                directory
            )
            unit.path to findings
        }
    }

    @Test
    fun GIVEN_recorded_trees_WHEN_hashed_THEN_match_the_recorded_hashes() {
        assertEquals(TREE_HASHES, TREES.map(::hashBlock))
    }

    @Test
    fun GIVEN_skill_document_WHEN_its_kotlin_trees_are_hashed_THEN_match_the_recorded_hashes() {
        val documents = System.getProperty(DOCUMENTS_PROPERTY)
        assumeTrue(documents != null, "Set -D$DOCUMENTS_PROPERTY=<path to .claude> to compare with the skill")
        val skill = File(documents, "skills/package-layout/SKILL.md").readText()

        assertEquals(TREE_HASHES, readKotlinBlocks(skill).map(::hashBlock))
    }

    @Test
    fun GIVEN_plugin_of_one_context_with_the_vote_capability_WHEN_each_unit_is_checked_THEN_reports_nothing(
        @TempDir directory: File
    ) {
        val case = TreeCase(
            units = SkillTreeReader("plugin", PLUGIN_UNIT_KINDS).read(listOf(PLUGIN_TREE, PLUGIN_VOTE_TREE)),
            dependencies = PLUGIN_DEPENDENCIES,
            entryPointWords = setOf(
                "command",
                "component",
                "context",
                "error",
                "event",
                "gui",
                "migration",
                "slot",
                "warmup"
            ),
            productKind = ProductKind.PLATFORM_PLUGIN,
            composeUnits = emptySet()
        )

        assertEquals(
            listOf(
                2,
                5,
                1,
                11,
                11,
                10,
                9,
                9,
                3,
                2
            ),
            case.units.map { unit -> unit.files.size }
        )
        assertEquals(
            listOf(
                "com.example.plugin.bukkit",
                "com.example.plugin.core.api",
                "com.example.plugin.core.bukkit",
                "com.example.plugin.data.dao",
                "com.example.plugin.data.exposed",
                "com.example.plugin.gui.api",
                "com.example.plugin.gui.bukkit",
                "com.example.plugin.command",
                "com.example.plugin.eventbukkit",
                "com.example.plugin.vote.api"
            ),
            case.units.map(SkillUnit::rootPackage)
        )
        assertEquals(
            case.units.associate { unit -> unit.path to emptyList<String>() },
            checkTree(case, directory)
        )
    }

    @Test
    fun GIVEN_plugin_of_several_capabilities_WHEN_each_unit_is_checked_THEN_reports_nothing(@TempDir directory: File) {
        val case = TreeCase(
            units = SkillTreeReader("plugin", CAPABILITIES_UNIT_KINDS).read(listOf(CAPABILITIES_TREE)),
            dependencies = CAPABILITIES_DEPENDENCIES,
            entryPointWords = setOf(
                "command",
                "event",
                "service"
            ),
            productKind = ProductKind.PLATFORM_PLUGIN,
            composeUnits = emptySet()
        )

        assertEquals(
            listOf(
                10,
                5,
                4
            ),
            case.units.map { unit -> unit.files.size }
        )
        assertEquals(
            listOf(
                "com.example.plugin.jail.api",
                "com.example.plugin.jail.bukkit",
                "com.example.plugin.home"
            ),
            case.units.map(SkillUnit::rootPackage)
        )
        assertEquals(
            case.units.associate { unit -> unit.path to emptyList<String>() },
            checkTree(case, directory)
        )
    }

    @Test
    fun GIVEN_app_WHEN_each_unit_is_checked_THEN_reports_nothing(@TempDir directory: File) {
        val case = TreeCase(
            units = SkillTreeReader("app", APP_UNIT_KINDS).read(listOf(APP_TREE)),
            dependencies = APP_DEPENDENCIES,
            entryPointWords = setOf("component", "composable"),
            productKind = ProductKind.APP,
            composeUnits = setOf("core/ui", "components/shop/payment/impl-ui")
        )

        assertEquals(
            listOf(
                2,
                5,
                3,
                3
            ),
            case.units.map { unit -> unit.files.size }
        )
        assertEquals(
            listOf(
                "com.example.app.core.ui",
                "com.example.app.shop.payment.api",
                "com.example.app.shop.payment.impl",
                "com.example.app.shop.payment.implui"
            ),
            case.units.map(SkillUnit::rootPackage)
        )
        assertEquals(
            case.units.associate { unit -> unit.path to emptyList<String>() },
            checkTree(case, directory)
        )
    }

    private companion object {
        const val DOCUMENTS_PROPERTY = "klibsRulesDocs"

        val PLUGIN_TREE = """
            instances/bukkit         ExamplePlugin.kt, di/RootModule.kt (builds the roots, lists their lifecycles)
            modules/core/api         …core.api
              settings/model/{PluginConfig, PluginPermission, PluginTranslation}.kt
              settings/migration/ConfigKeyMigration.kt
              di/CoreModule.kt
            modules/core/bukkit      …core.bukkit: di/BukkitModule.kt
            modules/data/dao         …data.dao (contracts, models, in-memory implementations; no DI module)
              rating/api/{RatingRepository, RatedUsersCache}.kt
              rating/memory/InMemoryRatedUsersCache.kt   production builds it
              rating/model/{NewUser, RatedUser, RatingStorageError, RatingType, UserRating}.kt
              rating/util/UserRatingExt.kt
            modules/data/exposed     …data.exposed
              rating/{ExposedRatingRepository, CacheRefreshingRatingRepository}.kt
              rating/RefreshingCache.kt                  the helper only CacheRefreshingRatingRepository uses
              rating/warmup/RatingCacheWarmUp.kt         seeds the cache at start
              rating/table/UserRatingTable.kt  rating/di/RatingStorageModule.kt
              database/RatingDatabase.kt  model/DatabaseConfig.kt  di/ExposedModule.kt
            modules/gui/api          …gui.api
              rating/change/usecase/{AddRatingUseCase, LowerKillerRatingUseCase}.kt
              rating/change/model/{AddRatingResult, RatingPlayer}.kt
              rating/change/util/PluginConfigExt.kt      isRatingMessageLengthAllowed(message)
              rating/list/component/{RatingListComponent, DefaultRatingListComponent}.kt
              rating/list/model/RatingSort.kt
              rating/list/di/RatingListModule.kt         takes RatedUsersCache as a parameter
              di/GuiModule.kt
            modules/gui/bukkit       …gui.bukkit
              rating/list/gui/RatingListGui.kt
              slot/{NavigationSlot, RatingSlot, SortSlot}.kt  context/SlotContext.kt  model/RatingSlotKey.kt
              util/ButtonExt.kt  di/GuiBukkitModule.kt
            modules/command          …command
              rating/command/{RatingCommandExecutor, RatingLiteralArgumentBuilder}.kt  rating/model/RatingCommand.kt
              error/CommandExceptionHandler.kt  model/UsageCommandException.kt  di/CommandsModule.kt
            modules/event-bukkit     …eventbukkit: event/{LoginListener, KillEventListener}.kt  di/EventModule.kt
        """.trimIndent()

        val PLUGIN_VOTE_TREE = """
            modules/vote/api         …vote.api: usecase/CastVoteUseCase.kt  di/VoteModule.kt
            modules/data/dao         vote/api/VoteRepository.kt  vote/model/Vote.kt
            modules/data/exposed     vote/ExposedVoteRepository.kt  vote/table/VoteTable.kt
            modules/command          vote/command/{VoteCommandExecutor, VoteLiteralArgumentBuilder}.kt  vote/model/VoteCommand.kt
            modules/gui/bukkit       slot/VoteSlot.kt                one file: its kind sits at the unit root
        """.trimIndent()

        val CAPABILITIES_TREE = """
            modules/jail/api         …jail.api
              usecase/{JailPlayerUseCase, FreeInmateUseCase}.kt
              model/{Jail, JailInmate, JailLocation, JailError}.kt
              api/JailRepository.kt
              file/FileJailRepository.kt              technology package: implementations directly
              memory/InMemoryJailRepository.kt
              di/JailModule.kt
            modules/jail/bukkit      …jail.bukkit
              command/JailCommand.kt  event/JailListener.kt  service/UnjailService.kt
              util/JailLocationExt.kt  di/JailBukkitModule.kt
            modules/home             …home: nothing else calls it, so one unit
              command/HomeCommand.kt  usecase/TeleportHomeUseCase.kt  model/Home.kt  di/HomeModule.kt
        """.trimIndent()

        val APP_TREE = """
            core/ui                         …core.ui: theme/AppTheme.kt  composable/ButtonComposable.kt
            components/shop/payment/api     …shop.payment.api
              offer/api/PaymentApi.kt  offer/model/Plan.kt  offer/stub/StubPaymentApi.kt
              offer/usecase/CheckSubscriptionUseCase.kt (interface)
              offer/component/OfferComponent.kt (interface: another unit opens the offer)
            components/shop/payment/impl    …shop.payment.impl
              offer/usecase/DefaultCheckSubscriptionUseCase.kt  offer/ktor/KtorPaymentApi.kt  offer/di/PaymentModule.kt
            components/shop/payment/impl-ui …shop.payment.implui
              offer/composable/OfferScreenComposable.kt  offer/component/DefaultOfferComponent.kt
              offer/di/PaymentUiModule.kt
        """.trimIndent()

        val TREES = listOf(
            PLUGIN_TREE,
            PLUGIN_VOTE_TREE,
            CAPABILITIES_TREE,
            APP_TREE
        )

        val TREE_HASHES = listOf(
            "2937578a96f8ec0d2f9a64561ea16ae58e924b139875567a04ffb98534285c99",
            "c82b57e07529544837a07722ef1b27b969c5491d74e477d84db8e3a04a5fcd7d",
            "a7dcaee7a21ea199e7f2a48e88eafdd6a82236b30dc873330baaaa94fd4f257f",
            "1af5cca635e98d7aab28d59a21204f7a8231e49d02013d09d4ce8bec72b8b2f7"
        )

        // The kinds `detectUnitKind` of the Gradle plugin gives these paths.
        val PLUGIN_UNIT_KINDS = mapOf(
            "instances/bukkit" to UnitKind.ENTRY_POINT,
            "modules/core/api" to UnitKind.CORE_API,
            "modules/core/bukkit" to UnitKind.CORE_PLATFORM,
            "modules/data/dao" to UnitKind.STORAGE_CONTRACT,
            "modules/data/exposed" to UnitKind.TECHNOLOGY,
            "modules/gui/api" to UnitKind.API,
            "modules/gui/bukkit" to UnitKind.PLATFORM,
            "modules/command" to UnitKind.CONTEXT,
            "modules/event-bukkit" to UnitKind.CONTEXT,
            "modules/vote/api" to UnitKind.API
        )

        // The tree gives command → vote/api, gui/bukkit → vote/api and vote/api → data/dao; the rest follows the
        // types each unit uses.
        val PLUGIN_DEPENDENCIES = mapOf(
            "instances/bukkit" to PLUGIN_UNIT_KINDS.keys.filter { path -> path.startsWith("modules/") },
            "modules/core/api" to emptyList(),
            "modules/core/bukkit" to listOf("modules/core/api"),
            "modules/data/dao" to emptyList(),
            "modules/data/exposed" to listOf("modules/core/api", "modules/data/dao"),
            "modules/gui/api" to listOf("modules/core/api", "modules/data/dao"),
            "modules/gui/bukkit" to listOf(
                "modules/core/api",
                "modules/gui/api",
                "modules/vote/api"
            ),
            "modules/command" to listOf(
                "modules/core/api",
                "modules/data/dao",
                "modules/vote/api"
            ),
            "modules/event-bukkit" to listOf("modules/core/api", "modules/data/dao"),
            "modules/vote/api" to listOf("modules/data/dao")
        )

        val CAPABILITIES_UNIT_KINDS = mapOf(
            "modules/jail/api" to UnitKind.API,
            "modules/jail/bukkit" to UnitKind.PLATFORM,
            "modules/home" to UnitKind.CONTEXT
        )

        // The tree gives jail/bukkit → jail/api, home → jail/api and jail/api → core/api only. core/api is not a unit
        // of this tree, no file of jail/api uses it, and the Konsist checks do not read the project dependencies.
        val CAPABILITIES_DEPENDENCIES = mapOf(
            "modules/jail/api" to emptyList(),
            "modules/jail/bukkit" to listOf("modules/jail/api"),
            "modules/home" to listOf("modules/jail/api")
        )

        // `impl-ui` is the IMPL_UI role's own name; `detectUnitKind` has no branch for it and gives CONTEXT.
        val APP_UNIT_KINDS = mapOf(
            "core/ui" to UnitKind.CORE_UI,
            "components/shop/payment/api" to UnitKind.API,
            "components/shop/payment/impl" to UnitKind.IMPL,
            "components/shop/payment/impl-ui" to UnitKind.IMPL_UI
        )

        // The tree gives impl-ui → payment/api, payment/impl and core/ui. Its shop/impl → payment/api names a unit
        // outside the tree; payment/impl depends on payment/api because DefaultCheckSubscriptionUseCase implements
        // CheckSubscriptionUseCase.
        val APP_DEPENDENCIES = mapOf(
            "core/ui" to emptyList(),
            "components/shop/payment/api" to emptyList(),
            "components/shop/payment/impl" to listOf("components/shop/payment/api"),
            "components/shop/payment/impl-ui" to listOf(
                "core/ui",
                "components/shop/payment/api",
                "components/shop/payment/impl"
            )
        )
    }
}
