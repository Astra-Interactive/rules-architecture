package ru.astrainteractive.klibs.rules.detekt.naming

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.api.config
import org.jetbrains.kotlin.psi.KtPackageDirective

/**
 * Reports a `package` directive that contains a layer, kind or filler word as one of its segments.
 *
 * `api` and `impl` are not in the default list: they are allowed as the segment of a build unit named so, and a
 * single file cannot tell where its unit's root package ends. `util` is allowed as a role inside a context.
 */
internal class ForbiddenPackageSegment(config: Config) : Rule(config, DESCRIPTION) {
    private val forbiddenSegments: List<String> by config(DEFAULT_FORBIDDEN_SEGMENTS)

    override fun visitPackageDirective(directive: KtPackageDirective) {
        super.visitPackageDirective(directive)
        val found = directive.qualifiedName
            .split('.')
            .filter { segment -> segment in forbiddenSegments }
        if (found.isEmpty()) return
        val message = "Rename the package segments ${found.joinToString { segment -> "'$segment'" }}: " +
            "a package is named for a capability or a role, never a layer or a kind."
        report(Finding(Entity.from(directive), message))
    }

    private companion object {
        @JvmField
        val DEFAULT_FORBIDDEN_SEGMENTS = listOf(
            "adapter",
            "aggregate",
            "application",
            "common",
            "controller",
            "dao",
            "domain",
            "dto",
            "entrypoint",
            "enum",
            "exception",
            "factory",
            "helpers",
            "inbound",
            "infrastructure",
            "internal",
            "misc",
            "outbound",
            "port",
            "presentation",
            "repository",
            "shared",
            "spi",
            "type",
            "ui",
            "usecase",
            "validation",
            "valueobject",
        )
        const val DESCRIPTION = "Package names follow the package-layout rule: no layer, kind or filler words."
    }
}
