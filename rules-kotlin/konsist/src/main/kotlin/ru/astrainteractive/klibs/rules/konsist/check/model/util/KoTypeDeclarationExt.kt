package ru.astrainteractive.klibs.rules.konsist.check.model.util

import com.lemonappdev.konsist.api.declaration.type.KoTypeDeclaration

private val TYPE_NAME = Regex("""\b[A-Z]\w*\b""")

/**
 * The simple name of the type without nullability, type arguments or qualifier: `Jail` for `model.Jail?`.
 */
internal val KoTypeDeclaration.simpleName: String
    get() = text.removeSuffix("?").substringBefore('<').substringAfterLast('.')

internal val KoTypeDeclaration.typeNames: Set<String>
    get() = TYPE_NAME.findAll(text).map(MatchResult::value).toSet()
