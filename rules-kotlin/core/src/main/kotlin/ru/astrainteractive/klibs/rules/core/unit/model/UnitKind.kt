package ru.astrainteractive.klibs.rules.core.unit.model

public enum class UnitKind(public val cliName: String) {
    CONTEXT("context"),
    API("api"),
    CONTRACT("contract"),
    ENTRY_POINT("entry-point"),
    TECHNOLOGY("technology"),
    FAKE("fake"),
    IMPL("impl"),
    PLATFORM("platform"),
    CORE_API("core-api"),
    CORE_PLATFORM("core-platform"),
    CORE_UI("core-ui"),
    STORAGE_CONTRACT("storage-contract")
}
