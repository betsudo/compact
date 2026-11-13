## Make ordinary usage behave ordinarily

A type-safe backend becomes ergonomic when familiar source expressions compile to predictable APIs. Constant-valued Vector projections, nested Map size expressions and direct Fold are examples. They are small at the source level and cross several implementation owners.

The work kept each gap bounded. Determine the Compact semantics, extend the checked representation if necessary, lower into structured Rust, exercise the generated consumer, and run applicable proof and refusal controls.

## Projection is not tuple selection

A source index into a fixed vector carries different type and result obligations from a tuple projection. Confusing them can make a generated value have the wrong shape even when one constant example works.

The accepted projection slice retained that distinction. Nested Map size and Fold then reused the checked foundation while adding their own ownership and effect requirements.

## Before and after · Index a Vector by a named constant

### Before · Use the position directly

A literal made the fixed position visible to the Rust backend. A source program using a constant expression could hit an unsupported projection even though the position was knowable during compilation.

~~~compact
// Illustrative literal-index version.
export pure circuit first(items: Vector<2, Field>): Field {
  return items[0];
}
~~~

### After · Let the compiler work out the position

~~~compact
// From the accepted fixture. Idx is an alias for Uint<8>.
export pure circuit first(items: Vector<2, Field>): Field {
  const i: Idx = 1;
  const index = i * 0;
  return items[index];
}
~~~

For input [7, 9], both versions select 7. The second version lets the programmer name and calculate the position. The same slice checks bounds and keeps dynamic indices outside this fixed-position feature.

## Wide multiplication by zero

A wide unsigned result cannot always be squeezed into a convenient native integer. Multiplying by zero still has a precise result type and output geometry.

PR #700 preserved that typed zero behavior. The review also retained a known limit around stateful zero-width results and frontend/ZKIR commitment agreement. The selected proof APIs declare Uint<8> results and establish those statements. They do not settle general zero-width proof readiness.

## Product and Vector widening

Many later slices concern a value that is valid in a wider expected context. The tricky cases involve nested products, aliases, literals and constructor loops. A widening has to preserve both value semantics and effects.

PR #703 reconciled an outer-Vector stack with the canonical backend. PR #704 added Linux proof coverage for eligible cases. PR #705 and its follow-ups qualified constructor Vector expressions and typed equality guards.

## Integration requirements

A logical slice includes its tests, documentation, review and integration result. Long and fragile CI motivated local work and measured job ownership. PR #716 split qualification into measured owners, while PR #723 reused an exact Nix compiler environment across consumers.

The goal was to reduce repeated preparation while retaining complete required gates. It was not permission to replace semantic tests with a faster badge.

## Result

More ordinary Compact programs reached typed generated Rust without ad hoc source rewrites. Each slice retained source/API/refusal qualification and proof/application evidence for selected applicable statements.

### Source trail

- [Constant-valued Vector indices PR #699](https://github.com/betsudo/compact/pull/699)
- [Wide Uint typed zero PR #700](https://github.com/betsudo/compact/pull/700)
- [Outer Vector reconciliation PR #703](https://github.com/betsudo/compact/pull/703)
- [Constructor Vector PR #705](https://github.com/betsudo/compact/pull/705)
- [Typed equality guards PR #719](https://github.com/betsudo/compact/pull/719)
- [Environment reuse PR #723](https://github.com/betsudo/compact/pull/723)
