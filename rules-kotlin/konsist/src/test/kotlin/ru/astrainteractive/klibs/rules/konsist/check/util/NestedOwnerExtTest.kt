@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.util

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class NestedOwnerExtTest {

    @Test
    fun GIVEN_class_with_properties_WHEN_types_listed_THEN_names_constructor_and_body_properties(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "com.example.jail.model/Jail.kt",
                """
                class Jail(val cell: Cell?, seed: Seed, var stage: List<Stage>) {
                    val name: String = ""
                    val untyped = 1
                }
                """
            )
            .main("com.example.jail.model/Cell.kt", "class Cell")
            .load(UnitKind.CONTEXT)
        val jail = sources.mainFiles
            .flatMap(KoFileDeclaration::classes)
            .single { declaration -> declaration.name == "Jail" }

        assertEquals(
            setOf(
                "Cell",
                "List",
                "Stage",
                "String"
            ),
            jail.propertyTypeNames()
        )
        assertEquals(2, sources.mainFiles.pathsNaming("Cell").size)
        assertEquals(emptySet(), sources.mainFiles.pathsNaming("Cel"))
    }
}
