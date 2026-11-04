@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.ownership.konsist

import com.lemonappdev.konsist.api.Konsist
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.io.File
import kotlin.test.assertEquals

class KotlinOwnershipReferencesTest {
    @Test
    fun GIVEN_unfinished_qualified_expression_WHEN_scanned_THEN_does_not_invent_reference(@TempDir root: File) {
        File(root, "Caller.kt").writeText("package y\nclass Caller")
        val file = Konsist.scopeFromExternalDirectory(root.absolutePath).files.single()
        val references = KotlinOwnershipReferences(setOf("x.Local"))

        assertEquals(emptySet(), references.resolve(file, "x.").names)
        assertEquals(emptySet(), references.resolve(file, "x.+").names)
        assertEquals(setOf("x.Local"), references.resolve(file, "x.Local").names)
    }
}
