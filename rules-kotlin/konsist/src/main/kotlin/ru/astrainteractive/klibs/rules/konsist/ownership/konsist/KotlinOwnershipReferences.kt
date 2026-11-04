package ru.astrainteractive.klibs.rules.konsist.ownership.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import org.jetbrains.kotlin.lexer.KotlinLexer
import org.jetbrains.kotlin.lexer.KtTokens
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName
import ru.astrainteractive.klibs.rules.konsist.ownership.model.OwnershipReferences

internal class KotlinOwnershipReferences(private val names: Set<String>) {
    private fun codeTokens(text: String): List<String> {
        val lexer = KotlinLexer()
        lexer.start(text)
        return buildList {
            while (lexer.tokenType != null) {
                if (lexer.tokenType == KtTokens.IDENTIFIER || lexer.tokenType == KtTokens.DOT) {
                    add(lexer.tokenText.removeSurrounding("`"))
                } else if (lexer.tokenType != KtTokens.WHITE_SPACE) {
                    add("")
                }
                lexer.advance()
            }
        }
    }

    private fun declaredPrefix(name: String): String? {
        return generateSequence(name) { current -> current.substringBeforeLast('.', "").ifEmpty { null } }
            .firstOrNull { current -> current in names }
    }

    private fun visibleNames(file: KoFileDeclaration): Map<String, Set<String>> {
        val visible = mutableMapOf<String, MutableSet<String>>()
        names.filter { name -> name.substringBeforeLast('.', "") == file.packageName }.forEach { name ->
            visible.getOrPut(name.substringAfterLast('.')) { mutableSetOf() }.add(name)
        }
        file.imports.forEach { import ->
            if (import.isWildcard) {
                names.filter { name -> name.substringBeforeLast('.') == import.name }.forEach { name ->
                    visible.getOrPut(name.substringAfterLast('.')) { mutableSetOf() }.add(name)
                }
            } else {
                val alias = import.alias
                val name = if (alias == null) import.name.substringAfterLast('.') else alias.name
                declaredPrefix(import.name)?.let { target ->
                    visible.getOrPut(name) { mutableSetOf() }.add(target)
                }
            }
        }
        return visible
    }

    private fun qualifiedNames(tokens: List<String>): Set<String> {
        return tokens.indices.mapNotNull { start ->
            if (tokens[start].isEmpty() || tokens[start] == ".") return@mapNotNull null
            val segments = mutableListOf(tokens[start])
            var index = start + 1
            while (index + 1 < tokens.size && tokens[index] == "." && tokens[index + 1].isNotEmpty()) {
                segments.add(tokens[index + 1])
                index += 2
            }
            declaredPrefix(segments.joinToString("."))
        }.toSet()
    }

    fun resolve(file: KoFileDeclaration, text: String): OwnershipReferences {
        val tokens = codeTokens(text)
        val identifiers = tokens.filter(String::isNotEmpty).toSet()
        val visible = visibleNames(file).filterKeys { name -> name in identifiers }.values
        val ambiguous = visible.filter { targets -> targets.size > 1 }.flatten().toSet()
        return OwnershipReferences(visible.flatten().toSet() + qualifiedNames(tokens), ambiguous)
    }
}
