// Site defaults; local article rules can override these settings.
#let render(body) = {
  set raw(theme: if sys.inputs.at("code-renderer", default: "typst") == "highlight-js" {
    none
  } else {
    sys.inputs.at("code-theme", default: auto)
  })
  if sys.inputs.at("math", default: "mathml") == "svg" {
    show math.equation: equation => if equation.block {
      html.frame(equation)
    } else {
      box(html.frame(equation))
    }
    body
  } else {
    body
  }
}
