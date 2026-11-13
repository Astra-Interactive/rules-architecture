@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.direction.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals

internal class KindImportDirectionRoleKindTest {

    private fun SourceTree.withVoteKinds(): SourceTree {
        return main("x.vote.vote.model/Vote.kt", "data class Vote(val id: Int)")
            .main("x.vote.vote.util/VoteExt.kt", "fun isRecent(): Boolean = true")
            .main("x.vote.vote.registry/VoteRegistry.kt", "internal class VoteRegistry")
            .main("x.vote.vote.api/VoteRepository.kt", "interface VoteRepository")
            .main("x.vote.vote.usecase/CastVoteUseCase.kt", "internal class CastVoteUseCase")
            .main("x.vote.vote.command/VoteCommand.kt", "internal class VoteCommand")
            .main("x.vote.vote.file/FileVoteRepository.kt", "internal class FileVoteRepository")
            .main("x.vote.vote.di/VoteModule.kt", "class VoteModule")
    }

    @Test
    fun GIVEN_role_kind_importing_every_kind_WHEN_checked_THEN_accepts_model_util_and_role_kinds_only(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .withVoteKinds()
            .main(
                "x.vote.vote.limiter/VoteCooldownLimiter.kt",
                """
                import x.vote.vote.api.VoteRepository
                import x.vote.vote.command.VoteCommand
                import x.vote.vote.di.VoteModule
                import x.vote.vote.file.FileVoteRepository
                import x.vote.vote.model.Vote
                import x.vote.vote.registry.VoteRegistry
                import x.vote.vote.usecase.CastVoteUseCase
                import x.vote.vote.util.isRecent

                internal class VoteCooldownLimiter
                """
            )
            .load(
                UnitKind.CONTEXT,
                rootPackage = "x.vote",
                vocabulary = ROLE_KIND_WORDS
            )

        val violations = KindImportDirection().check(sources)

        assertEquals(
            listOf(
                "x.vote.vote.api.VoteRepository",
                "x.vote.vote.command.VoteCommand",
                "x.vote.vote.di.VoteModule",
                "x.vote.vote.file.FileVoteRepository",
                "x.vote.vote.usecase.CastVoteUseCase",
            ),
            violations.map(Violation::subject).sorted()
        )
        assertEquals(
            "'vote/limiter' must not import x.vote.vote.api.VoteRepository from 'vote/api': " +
                "a role kind depends on model, util, role only.",
            violations.first { violation -> violation.subject == "x.vote.vote.api.VoteRepository" }.message
        )
    }

    @Test
    fun GIVEN_every_kind_importing_a_role_kind_WHEN_checked_THEN_rejects_model_contracts_and_util(
        @TempDir directory: File
    ) {
        val tree = SourceTree(directory)
            .main("x.vote.vote.limiter/VoteCooldownLimiter.kt", "internal class VoteCooldownLimiter")
        listOf(
            "model",
            "api",
            "util",
            "usecase",
            "command",
            "file",
            "stub",
            "di"
        ).forEach { kind ->
            tree.main(
                "x.vote.vote.$kind/Consumer.kt",
                "import x.vote.vote.limiter.VoteCooldownLimiter\n\ninternal class Consumer"
            )
        }
        val sources = tree.load(
            UnitKind.CONTEXT,
            rootPackage = "x.vote",
            vocabulary = ROLE_KIND_WORDS
        )

        assertEquals(
            listOf(
                "api/Consumer.kt",
                "model/Consumer.kt",
                "util/Consumer.kt"
            ),
            KindImportDirection().check(sources).map { violation ->
                violation.path.substringAfter("x/vote/vote/")
            }.sorted()
        )
    }

    @Test
    fun GIVEN_entry_point_violation_in_a_project_with_role_kinds_WHEN_reported_THEN_lists_role_kinds_as_allowed(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.jail.jail.command",
                "JailCommand.kt",
                "import x.jail.jail.di.JailModule\n\nclass JailCommand"
            )
            .main(
                "x.jail.jail.di",
                "JailModule.kt",
                "class JailModule"
            )
            .load(
                UnitKind.CONTEXT,
                unitPath = ":jail",
                vocabulary = ROLE_KIND_WORDS
            )

        assertEquals(
            listOf(
                "'jail/command' must not import x.jail.jail.di.JailModule from 'jail/di': " +
                    "an entry point kind depends on model, contract, util, usecase, theme, role only."
            ),
            KindImportDirection().check(sources).map(Violation::message)
        )
    }

    private companion object {
        val ROLE_KIND_WORDS = SourceTree.NO_PROJECT_WORDS.copy(roleKinds = setOf("limiter", "registry"))
    }
}
