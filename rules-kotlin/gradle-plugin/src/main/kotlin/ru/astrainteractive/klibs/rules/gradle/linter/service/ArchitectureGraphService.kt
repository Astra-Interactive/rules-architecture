package ru.astrainteractive.klibs.rules.gradle.linter.service

import org.gradle.api.provider.ListProperty
import org.gradle.api.services.BuildService
import org.gradle.api.services.BuildServiceParameters

internal abstract class ArchitectureGraphService : BuildService<ArchitectureGraphService.Parameters> {
    interface Parameters : BuildServiceParameters {
        val unitMetadata: ListProperty<String>
    }
}
