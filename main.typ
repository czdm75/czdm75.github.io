#let content-prefix = "content/"

// This list is generated externally because Typst cannot enumerate directories.
#let content-files = (
  "content/cs/io/java-nio-1.typst",
  "content/cs/io/java-nio-2.typst",
  "content/math/max-likelihood-estimation.typ",
  "content/pl/lambda-calculus-y-combinator.typ",
  "content/pl/monad-scala-perspective.typ",
  "content/pl/scala-currying-partials.typ",
)

#let article-paths = (
  content-files
    .filter(path => path.starts-with(content-prefix))
    .filter(path => path.ends-with(".typ") or path.ends-with(".typst"))
    .sorted()
)

#for article-path in article-paths {
  let relative-path = article-path.slice(content-prefix.len())
  let parts = relative-path.split("/")
  let filename = parts.last()
  let slug = filename.split(".").slice(0, -1).join(".")
  let url = (parts.slice(0, -1) + (slug, "index.html")).join("/")
  import article-path as article

  document(
    url,
    title: article.meta.title,
  )[
    #set text(lang: article.meta.lang)
    #include article-path
  ]
}
