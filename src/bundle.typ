#for item in articles {
  import item.source as article
  let meta = dictionary(article).at("meta", default: (:))
  document(item.output, title: meta.at("title", default: none))[
    #set text(lang: meta.at("lang", default: "zh"))
    #show: render
    #include item.source
    #metadata(meta) <ssg-page>
  ]
}
