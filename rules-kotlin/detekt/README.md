# detekt

Detekt rules that report the architecture-rule violations visible in a single Kotlin file.

## Annotation placement

Place each declaration annotation on its own line above the declaration, including annotations on
constructor and function parameters. Put consecutive annotations on separate lines. The declaration's
modifiers and keywords follow on a later line than the final annotation's closing token.
This convention covers declaration annotations, including use-site targets such as `@get:` and
`@field:`. Type-use and expression annotations are outside this rule's scope.

```kotlin
data class Entry(
    @SerialName("index")
    val index: Int,
)
```

`AnnotationOnSeparateLine` reports inline declaration annotations and tells the agent to move them
above the declaration. It performs no automatic correction.
