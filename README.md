# NameParser for Rust

This library is an approximation of my [`NameParserSharp` C# library](https://github.com/aeshirey/NameParserSharp) used to parse peoples' names into constituent parts. For example:

```rust
let p = PersonName::parse("Johannes Diderik van der Waals").unwrap();
assert_eq!(p.first, "Johannes");
assert_eq!(p.middle, "Diderik");
assert_eq!(p.last, "van der Waals");
assert_eq!(p.suffix, "");
```

The original project and therefore my direct C# port are LGPL-encumbered. This implementation is inspired by but not a port of the C#; therefore, I believe it is not LGPL-encumbered. As such, I am releasing this under the MIT license.

This version does not have all the functionality of the other libraries. Notably:

* It only handles parenthetical nicknames such as `James (Jimmy) Carter`
* The set of prefixes, suffixes, conjunctions, etc., is more limited.
* It doesn't handle parsing out multiple persons from a single input. For example, `John D. and Katherine T. MacArthur` doesn't parse out to become two individuals.
