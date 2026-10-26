package ru.astrainteractive.klibs.rules.konsist.check.model

import com.lemonappdev.konsist.api.declaration.KoBaseDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration

internal sealed interface ViolationOrigin {
    val path: String
}

internal data class DeclarationOrigin(
    val declaration: KoBaseDeclaration,
    override val path: String
) : ViolationOrigin

internal data class FileOrigin(
    val file: KoFileDeclaration,
    override val path: String
) : ViolationOrigin

internal data class ExternalOrigin(override val path: String) : ViolationOrigin
