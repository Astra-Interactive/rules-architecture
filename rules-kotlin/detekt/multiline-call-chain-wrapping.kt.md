# Multiline call chains

Once a call chain has a line break between its operations, place each call after the first call,
each safe cast (`as?`), and each Elvis operator (`?:`) on a new line. Put the operator at the start
of the continuation line. The first call can share a line with its receiver, including a qualified
property path leading to that receiver.

```kotlin
val principal = principals.getFlow()
    .first { state -> state != Principal.Loading }
    as? Principal.Token
    ?: return
```

A chain contains at least one call. Its connected receiver expressions, safe casts and left operands
of Elvis expressions form the chain. A line break around any connecting operator activates this rule,
including a break at a property access, safe cast or Elvis operator. Property accesses can share lines
with adjacent operations. Qualified names within a cast's type remain together.

A single-line chain can stay on one line. Line breaks inside call arguments or lambda bodies alone do
not activate this rule. Nested expressions, including parenthesized expressions and an Elvis operator's
right operand, form independent chains. Casts and Elvis expressions without calls are outside its scope.

`MultilineCallChainWrapping` reports each call, safe cast or Elvis operator that needs a preceding line
break. It performs no automatic correction.
