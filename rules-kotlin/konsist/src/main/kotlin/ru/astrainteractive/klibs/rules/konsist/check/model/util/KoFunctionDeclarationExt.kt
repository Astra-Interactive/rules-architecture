package ru.astrainteractive.klibs.rules.konsist.check.model.util

import com.lemonappdev.konsist.api.declaration.KoFunctionDeclaration
import com.lemonappdev.konsist.api.declaration.KoParameterDeclaration
import com.lemonappdev.konsist.api.declaration.KoTypeParameterDeclaration
import com.lemonappdev.konsist.api.declaration.type.KoTypeDeclaration

private val FUN_KEYWORD = Regex("""\bfun\b""")

private val CONTEXT_LIST = Regex("""\bcontext\s*\((.*)\)""", RegexOption.DOT_MATCHES_ALL)

/**
 * The text of every type the signature names: the context parameters, the bounds of the type parameters, the
 * receiver and the parameters. Konsist reads no context parameter, so the list is cut from the text before `fun`.
 */
internal fun KoFunctionDeclaration.signatureTypeTexts(): List<String> {
    val header = text.removePrefix(kDoc?.text.orEmpty()).split(FUN_KEYWORD, limit = 2).first()
    val contextList = CONTEXT_LIST.find(header)?.groupValues?.get(1)
    val bounds = typeParameters.flatMap(KoTypeParameterDeclaration::upperBounds)
    val types = bounds + listOfNotNull(receiverType) + parameters.map(KoParameterDeclaration::type)
    return listOfNotNull(contextList) + types.map(KoTypeDeclaration::text)
}
