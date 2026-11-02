@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.util

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.RequiresAnalysisApi
import dev.detekt.api.Rule
import dev.detekt.api.RuleName
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.jetbrains.kotlin.psi.KtProperty
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class KtPropertyExtTest {
    @Test
    fun GIVEN_constructed_injected_aliased_and_factory_values_WHEN_inspected_THEN_only_construction_proves_ownership() {
        val code = """
            package x.di

            class Child(val value: Int)
            fun createChild(): Child = Child(1)
            fun Child(): String = "a factory named like a constructor"
            fun <T> otherLazy(block: () -> T): Lazy<T> = lazy(block)

            class Parent(injected: x.other.Resource) {
                val direct = x.other.Resource()
                val deferred by lazy { x.other.Resource() }
                val borrowed = injected
                val alias = direct
                val factory = createChild()
                val uppercaseFactory = Child()
                val constant = 1
                val property = injected.name
                val arithmetic = 1 + 2
                val libraryFactory = x.other.createResource()
                val deferredFactory by lazy { createChild() }
                val opaqueDelegate by otherLazy { x.other.Resource() }
                val multiStatement by lazy { println("constructing"); x.other.Resource() }
                val constantDelegate by lazy { 1 }
            }
        """.trimIndent()
        val resource = """
            package x.other

            class Resource { val name = "resource" }
            fun createResource(): Resource = Resource()
        """.trimIndent()
        val rule = object : Rule(Config.empty, "Observe ownership evidence"), RequiresAnalysisApi {
            override val ruleName = RuleName("OwnershipEvidence")

            override fun visitProperty(property: KtProperty) {
                super.visitProperty(property)
                if (property.ownedConstructionOrNull() == null) return
                report(Finding(Entity.from(property), property.name.orEmpty()))
            }
        }
        val findings = rule.lintWithContext(
            createEnvironment(),
            code,
            resource
        )

        assertEquals(listOf("direct", "deferred"), findings.map(Finding::message))
    }
}
