package ru.astrainteractive.klibs.rules.konsist.check.model.util

import com.lemonappdev.konsist.api.declaration.type.KoTypeDeclaration

private val TYPE_NAME = Regex("""\b[A-Z]\w*\b""")

/**
 * The simple name of the type without nullability, type arguments or qualifier: `Jail` for `model.Jail?`.
 */
internal val KoTypeDeclaration.simpleName: String
    get() = text.removeSuffix("?").substringBefore('<').substringAfterLast('.')

/**
 * The name of the type and of the types it is nested in, outermost first, without package, nullability or type
 * arguments: `[TagDTO, Real]` for `model.TagDTO.Real?`.
 */
internal val KoTypeDeclaration.typePath: List<String>
    get() {
        val segments = text.removeSuffix("?").substringBefore('<').split('.')
        return segments.dropLast(1).dropWhile { segment -> segment.first().isLowerCase() } + segments.last()
    }

internal val KoTypeDeclaration.typeNames: Set<String>
    get() = TYPE_NAME.findAll(text).map(MatchResult::value).toSet()
