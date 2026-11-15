= Bare document

== A child heading

```rust
fn main() { println!("hello"); }
```

#metadata((kind: "example", enabled: true)) <extension-example>

`inline`

#raw("first\nsecond", block: true, lang: "text")

#show raw.where(lang: "custom"): it => [Rendered replacement]
#raw("original source", lang: "custom")

#metadata(datetime.today().display("[year]-[month]-[day]")) <today>

Inline math: $x^2 + y^2$.

$ sum_(i=1)^n i $
