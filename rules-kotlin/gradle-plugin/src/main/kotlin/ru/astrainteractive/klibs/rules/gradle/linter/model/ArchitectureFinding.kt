package ru.astrainteractive.klibs.rules.gradle.linter.model

internal data class ArchitectureFinding(
    val rule: String,
    val path: String,
    val subject: String,
    val message: String
) {
    /**
     * Leaves the message out, so rewording a rule keeps the baseline valid.
     */
    val baselineKey: String
        get() = listOf(rule, path, subject).joinToString(separator = FIELD_SEPARATOR)

    fun describe(): String = "$path: $message [$rule]"

    companion object {
        const val FIELD_SEPARATOR = "\t"
        private const val FIELD_COUNT = 4

        /**
         * Parses `rule<TAB>path<TAB>subject<TAB>message`.
         */
        fun parse(line: String): ArchitectureFinding {
            val fields = line.split(FIELD_SEPARATOR, limit = FIELD_COUNT)
            require(fields.size == FIELD_COUNT) { "Malformed architecture report line: '$line'" }
            return ArchitectureFinding(rule = fields[0], path = fields[1], subject = fields[2], message = fields[3])
        }
    }
}
