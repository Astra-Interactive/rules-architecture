package ru.astrainteractive.klibs.rules.gradle.linter.util

import org.gradle.api.artifacts.ProjectDependency
import ru.astrainteractive.klibs.rules.core.unit.model.UnitReference

/** Uses the Gradle-generated project accessor without keeping a Project in the lint task inputs. */
public fun ProjectDependency.toUnitReference(): UnitReference = UnitReference(path)
