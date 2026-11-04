package ru.astrainteractive.klibs.rules.konsist.ownership.konsist

import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.ownership.model.OwnershipAssessment
import ru.astrainteractive.klibs.rules.konsist.ownership.model.OwnershipDeclaration

internal class ApiOwnershipReport {
    private fun isImplementation(kind: UnitKind): Boolean {
        return when (kind) {
            UnitKind.IMPL, UnitKind.IMPL_UI, UnitKind.PLATFORM, UnitKind.TECHNOLOGY, UnitKind.CONTEXT -> true
            else -> false
        }
    }

    private fun findDependents(
        name: String,
        declarations: List<OwnershipDeclaration>,
        visited: MutableSet<String>
    ): List<OwnershipDeclaration> {
        if (!visited.add(name)) return emptyList()
        return declarations.filter { declaration -> name in declaration.references && !declaration.wiring }
            .flatMap { declaration ->
                val indirect = if (declaration.unit.context.kind == UnitKind.API) {
                    findDependents(
                        declaration.name,
                        declarations,
                        visited
                    )
                } else {
                    emptyList()
                }
                indirect + declaration
            }.distinct()
    }

    private fun assess(
        name: String,
        variants: List<OwnershipDeclaration>,
        declarations: List<OwnershipDeclaration>
    ): OwnershipAssessment {
        val owners = variants.map { declaration -> declaration.unit.path }.toSet()
        val dependents = findDependents(
            name,
            declarations,
            mutableSetOf()
        )
        val consumers = dependents.map { declaration -> declaration.unit.path }.toSet() - owners
        val consumer = consumers.singleOrNull()
        val owner = owners.singleOrNull()
        val sameParent = owner?.substringBeforeLast(':') == consumer?.substringBeforeLast(':')
        val sibling = owner != null && consumer != null && sameParent
        val implementation = dependents.any { declaration ->
            declaration.unit.path == consumer && isImplementation(declaration.unit.context.kind)
        }
        val ambiguous = dependents.any { declaration -> declaration.ambiguousReferences.isNotEmpty() }
        val published = variants.any { declaration -> declaration.unit.context.publishedLibrary }
        val status = when {
            published -> OwnershipAssessment.Status.EXTERNAL
            owner == null || ambiguous -> OwnershipAssessment.Status.UNCERTAIN
            consumers.isEmpty() -> OwnershipAssessment.Status.UNKNOWN
            sibling && implementation -> OwnershipAssessment.Status.CANDIDATE
            else -> OwnershipAssessment.Status.RETAIN
        }
        return OwnershipAssessment(
            name,
            status,
            variants.map(OwnershipDeclaration::location).sorted(),
            consumers.sorted(),
            declarations.filter { declaration -> name in declaration.references }
                .sortedBy(OwnershipDeclaration::location)
        )
    }

    private fun StringBuilder.appendAssessment(assessment: OwnershipAssessment) {
        appendLine()
        appendLine("${assessment.status} ${assessment.name}")
        appendLine("  declared: ${assessment.locations.joinToString()}")
        val consumers = assessment.consumers.joinToString().ifEmpty { "none found" }
        appendLine("  observed consumers: $consumers")
        val explanation = when (assessment.status) {
            OwnershipAssessment.Status.CANDIDATE ->
                "review placement with $consumers; one sibling consumer is not proof."
            OwnershipAssessment.Status.EXTERNAL ->
                "published API: external consumers are not visible in this build."
            OwnershipAssessment.Status.UNCERTAIN ->
                "ambiguous references or duplicate declaration owners; resolve them before deciding placement."
            OwnershipAssessment.Status.UNKNOWN ->
                "no non-DI production consumer found; this does not prove unused code."
            OwnershipAssessment.Status.RETAIN ->
                "observed references do not identify a sole sibling implementation owner."
        }
        appendLine("  $explanation")
        assessment.evidence.forEach { declaration ->
            val kind = if (declaration.wiring) "DI" else "REFERENCE"
            appendLine("  $kind ${declaration.unit.path} ${declaration.location} (${declaration.name})")
        }
    }

    fun render(declarations: List<OwnershipDeclaration>, limitations: List<String>): String {
        return buildString {
            appendLine("API ownership review — advisory, not architecture violations.")
            appendLine("Candidates require a responsibility and signature review; no automatic moves.")
            appendLine("Source references are conservative lexical evidence, not compiler-resolved calls.")
            appendLine("Only supplied production sources participate; tests and testFixtures do not justify API.")
            appendLine("DI references are shown separately and do not establish shared behaviour.")
            appendLine(
                "No candidates does not prove compliance; external, generated or unregistered consumers may exist."
            )
            limitations.distinct().sorted().forEach { limitation -> appendLine("LIMITATION $limitation") }
            declarations.filter { declaration -> declaration.unit.context.kind == UnitKind.API }
                .groupBy(OwnershipDeclaration::name).toSortedMap().forEach { (name, variants) ->
                    appendAssessment(
                        assess(
                            name,
                            variants,
                            declarations
                        )
                    )
                }
        }
    }
}
