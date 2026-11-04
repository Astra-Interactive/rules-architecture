@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.ownership.konsist

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.ownership.model.OwnershipDeclaration
import ru.astrainteractive.klibs.rules.konsist.ownership.model.OwnershipUnit
import java.io.File
import kotlin.test.assertContains
import kotlin.test.assertFalse

class ApiOwnershipReportTest {
    private fun declaration(
        name: String,
        unit: String = ":voice:api",
        references: Set<String> = emptySet(),
        wiring: Boolean = false,
        kind: UnitKind = if (unit.endsWith(":api")) UnitKind.API else UnitKind.PLATFORM,
        published: Boolean = false
    ): OwnershipDeclaration {
        val context = UnitContext(
            kind,
            null,
            published,
            emptySet(),
            emptyList()
        )
        return OwnershipDeclaration(
            name,
            OwnershipUnit(
                unit,
                File("."),
                context
            ),
            "$name.kt:1",
            references,
            wiring,
            emptySet()
        )
    }

    @Test
    fun GIVEN_discord_local_usecases_gateway_and_errors_WHEN_reviewed_THEN_reports_sibling_candidates() {
        val declarations = listOf(
            declaration("CreateMemberVoiceChannelUseCase", references = setOf("VoiceChannelGateway")),
            declaration("DeleteEmptyVoiceChannelUseCase", references = setOf("VoiceChannelGateway")),
            declaration("VoiceChannelGateway", references = setOf("GuildId")),
            declaration("GuildId"),
            declaration("CompletionError", ":ai:api"),
            declaration("ChatHistoryError", ":ai:api"),
            declaration(
                "VoiceChannelFeature",
                ":voice:jda",
                setOf("CreateMemberVoiceChannelUseCase", "DeleteEmptyVoiceChannelUseCase")
            ),
            declaration(
                "KtorCompletionGateway",
                ":ai:ktor",
                setOf("CompletionError")
            ),
            declaration(
                "ExposedChatHistoryRepository",
                ":ai:exposed",
                setOf("ChatHistoryError")
            )
        )

        val report = ApiOwnershipReport().render(declarations, emptyList())

        listOf(
            "CreateMemberVoiceChannelUseCase",
            "DeleteEmptyVoiceChannelUseCase",
            "VoiceChannelGateway",
            "GuildId",
            "CompletionError",
            "ChatHistoryError"
        ).forEach { name ->
            assertContains(report, "CANDIDATE $name")
        }
        assertContains(report, "REFERENCE :voice:jda VoiceChannelFeature.kt:1")
    }

    @Test
    fun GIVEN_shared_repository_and_transitive_signature_models_WHEN_reviewed_THEN_retains_the_closure() {
        val declarations = listOf(
            declaration("VoiceChannelRepository", references = setOf("TrackedVoiceChannel")),
            declaration("TrackedVoiceChannel", references = setOf("MemberId")),
            declaration("MemberId", references = setOf("TrackedVoiceChannel")),
            declaration(
                "Caller",
                ":voice:jda",
                setOf("VoiceChannelRepository")
            ),
            declaration(
                "Implementation",
                ":voice:exposed",
                setOf("VoiceChannelRepository")
            )
        )

        val report = ApiOwnershipReport().render(declarations, emptyList())

        assertContains(report, "RETAIN VoiceChannelRepository")
        assertContains(report, "RETAIN TrackedVoiceChannel")
        assertContains(report, "RETAIN MemberId")
        assertFalse(report.contains("CANDIDATE "))
    }

    @Test
    fun GIVEN_foreign_api_and_published_consumers_WHEN_reviewed_THEN_does_not_suggest_local_moves() {
        val declarations = listOf(
            declaration("ForeignUseCase"),
            declaration("ApiDependency"),
            declaration("ExternalLibraryApi", published = true),
            declaration(
                "Caller",
                ":other:impl",
                setOf("ForeignUseCase")
            ),
            declaration(
                "OtherApi",
                ":other:api",
                setOf("ApiDependency")
            )
        )

        val report = ApiOwnershipReport().render(declarations, emptyList())

        assertContains(report, "RETAIN ForeignUseCase")
        assertContains(report, "RETAIN ApiDependency")
        assertContains(report, "EXTERNAL ExternalLibraryApi")
    }

    @Test
    fun GIVEN_only_di_no_usages_or_duplicate_owners_WHEN_reviewed_THEN_reports_uncertainty() {
        val declarations = listOf(
            declaration("Factory"),
            declaration("Unknown"),
            declaration("Duplicate"),
            declaration("Duplicate", ":other:api"),
            declaration(
                "RootModule",
                ":app",
                setOf("Factory"),
                wiring = true
            )
        )

        val report = ApiOwnershipReport().render(declarations, listOf("Java sources excluded"))

        assertContains(report, "UNKNOWN Factory")
        assertContains(report, "DI :app RootModule.kt:1")
        assertContains(report, "UNKNOWN Unknown")
        assertContains(report, "UNCERTAIN Duplicate")
        assertContains(report, "LIMITATION Java sources excluded")
    }

    @Test
    fun GIVEN_impl_ui_and_expect_actual_variants_WHEN_reviewed_THEN_merges_consumers() {
        val declarations = listOf(
            declaration("UiModel"),
            declaration("UiModel"),
            declaration(
                "Screen",
                ":voice:ui",
                setOf("UiModel"),
                kind = UnitKind.IMPL_UI
            )
        )

        val report = ApiOwnershipReport().render(declarations, emptyList())

        assertContains(report, "CANDIDATE UiModel")
        assertContains(report, "observed consumers: :voice:ui")
    }

    @Test
    fun GIVEN_sibling_roles_WHEN_reviewed_THEN_only_implementation_roles_are_candidates() {
        val roles = listOf(
            UnitKind.CONTEXT,
            UnitKind.IMPL,
            UnitKind.TECHNOLOGY,
            UnitKind.ENTRY_POINT,
            UnitKind.API
        )
        roles.forEach { role ->
            val declarations = listOf(
                declaration("Operation"),
                declaration(
                    "Caller",
                    ":voice:sibling",
                    setOf("Operation"),
                    kind = role
                )
            )

            val report = ApiOwnershipReport().render(declarations, emptyList())

            val status = if (role == UnitKind.ENTRY_POINT || role == UnitKind.API) "RETAIN" else "CANDIDATE"
            assertContains(report, "$status Operation")
        }
    }
}
