package ru.astrainteractive.klibs.rules.konsist.ownership.model

internal data class OwnershipAssessment(
    val name: String,
    val status: Status,
    val locations: List<String>,
    val consumers: List<String>,
    val evidence: List<OwnershipDeclaration>
) {
    enum class Status {
        CANDIDATE,
        EXTERNAL,
        UNCERTAIN,
        UNKNOWN,
        RETAIN
    }
}
