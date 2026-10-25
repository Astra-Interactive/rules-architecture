@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class StateTypePlacementTest {

    @Test
    fun GIVEN_nested_and_misplaced_states_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "com.example.splash.viewmodel/SplashViewModel.kt",
                """
                class SplashViewModel {
                    sealed interface SplashPaywallState
                }

                data class RatingPlayersState(val count: Int)
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "com.example")

        val messages = StateTypePlacement().check(sources).map(Violation::message)

        assertEquals(
            listOf(
                "Move SplashPaywallState to its own file in model/: a screen state is never nested in its view " +
                    "model or state holder.",
                "Move RatingPlayersState to the model package of its level: the view reads the screen state.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_states_in_model_and_compose_names_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "com.example.splash.model/SplashPaywallState.kt",
                """
                sealed interface SplashPaywallState {
                    object Finished : SplashPaywallState
                }
                """
            )
            .main(
                "com.example.splash.composable/SplashComposable.kt",
                """
                class Holder {
                    class MutableState
                    class UiState
                }
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "com.example")

        assertEquals(emptyList(), StateTypePlacement().check(sources))
    }
}
