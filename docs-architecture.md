# Documentation Architecture: The Knowledge Graph

> Context note: this is a documentation strategy note for how `_cmd` docs should
> evolve over time. It includes future structure, not just what is present
> today.

For `_cmd` to scale into a robust terminal dashboard, its documentation should
be treated with the same architectural rigor as its code.

## 1. The Diataxis Framework

Documentation can be grouped into four categories:

1. Tutorials: step-by-step onboarding
2. How-to guides: goal-oriented tasks
3. Reference: exact technical facts and APIs
4. Explanation: architecture, rationale, and tradeoffs

## 2. Architecture Decision Records

Major technical decisions should eventually be recorded under a dedicated
architecture decision area so future contributors understand why choices were
made.

## 3. Code as Documentation

Public Rust APIs should use rustdoc comments where helpful, and examples should
stay runnable where possible.

## 4. Single Source of Truth

- `README.md` should remain the top-level entrypoint.
- Detailed architecture and roadmap material can live in `docs/`.
- Domain-specific docs should stay near their subject rather than turning into
  one giant markdown file.
