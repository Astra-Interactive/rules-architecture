package ru.astrainteractive.klibs.rules.detekt.util

private val WORD = Regex("[a-z]+")

private val WORD_START = Regex("(?=[A-Z])|_")

private val FILLER_WORDS = setOf(
    "a",
    "an",
    "for",
    "given",
    "its",
    "of",
    "the",
    "this",
    "to"
)

/**
 * The lowercase words of KDoc prose, without articles and other filler words.
 */
internal fun wordsOf(text: String): List<String> {
    return WORD.findAll(text.lowercase()).map(MatchResult::value).filterNot(FILLER_WORDS::contains).toList()
}

/**
 * The words of a camel-case or snake-case name: `userId` gives `user`, `id`.
 */
internal fun nameWordsOf(name: String): List<String> {
    return name.split(WORD_START).flatMap(::wordsOf)
}

private fun isSameWord(word: String, nameWord: String): Boolean {
    return word.startsWith(nameWord) || nameWord.startsWith(word)
}

/**
 * Whether every word of [content] is a word of the name, written whole or shortened (`identifier` for `id`).
 */
internal fun isCoveredBy(content: List<String>, nameWords: List<String>): Boolean {
    return content.all { word -> nameWords.any { nameWord -> isSameWord(word, nameWord) } }
}
