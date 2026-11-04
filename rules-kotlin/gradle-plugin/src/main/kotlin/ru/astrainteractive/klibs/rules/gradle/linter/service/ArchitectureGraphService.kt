package ru.astrainteractive.klibs.rules.gradle.linter.service

import org.gradle.api.provider.ListProperty
import org.gradle.api.provider.Property
import org.gradle.api.services.BuildService
import org.gradle.api.services.BuildServiceParameters
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind

internal abstract class ArchitectureGraphService : BuildService<ArchitectureGraphService.Parameters> {
    interface Parameters : BuildServiceParameters {
        val unitMetadata: ListProperty<String>
        val ownershipMetadata: ListProperty<String>
        val productKind: Property<ProductKind>
    }
}
