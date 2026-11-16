// Expand a user-authored summary into selected articles and navigation labels.
#let select-summary(summary, articles) = {
  let walk(items, groups) = {
    assert(type(items) == array, message: "summary children must be an array")
    let selected = ()
    for (index, entry) in items.enumerate() {
      let key = "entry-" + str(index)
      let item = if type(entry) == str { (path: entry) } else { entry }
      assert(type(item) == dictionary, message: "summary entries must be paths or dictionaries")
      let path = item.at("path", default: none)
      let title = item.at("title", default: none)
      let collapsed = item.at("collapsed", default: false)
      assert(type(collapsed) == bool, message: "summary collapsed must be a boolean")
      assert(title == none or type(title) == str, message: "summary title must be a string")
      assert(path == none or type(path) == str, message: "summary path must be a string")
      if "children" in item {
        let name = if title != none { title } else {
          assert(path != none, message: "summary group needs a title or path")
          path.trim(at: end, "/").split("/").last()
        }
        selected += walk(item.children, groups + ((key: key, title: name, collapsed: collapsed),))
      } else {
        assert(path != none, message: "summary entry needs a path or children")
        let matches = articles.filter(article => article.path == path)
        if matches.len() > 0 {
          selected.push((..matches.first(), navigation: (groups: groups, title: title)))
        } else {
          let prefix = if path == "." { "" } else { path.trim(at: end, "/") + "/" }
          let descendants = articles.filter(article => article.path.starts-with(prefix))
          assert(descendants.len() > 0, message: "summary path not found or empty: " + path)
          let name = if title != none { title } else { path.trim(at: end, "/").split("/").last() }
          let parent = if path == "." and title == none { groups } else { groups + ((key: key, title: name, collapsed: collapsed),) }
          for article in descendants {
            let parts = article.path.slice(prefix.len()).split("/")
            selected.push((..article, navigation: (
              groups: parent + parts.slice(0, -1).map(part => (key: "directory-" + part, title: part, collapsed: false)), title: none,
            )))
          }
        }
      }
    }
    selected
  }
  let selected = walk(summary, ())
  let paths = selected.map(article => article.path)
  assert(paths.dedup().len() == paths.len(), message: "summary selects an article more than once")
  selected
}
