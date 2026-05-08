#let template(
  titre: "Titre",
  cours: "Cours",
  code: "GEN490",
  auteurs: (),
  auteurs_footer: true,
  date: "0 juin 2020",
  body,
) = {
  // Set the document's basic properties.
  set document(author: auteurs.map(a => a.nom), title: titre)
  set page(
    paper: "us-letter",
    margin: (x: 3cm, top: 2cm, bottom: if auteurs_footer {
      (auteurs.len() * 1cm) + 1cm
    } else { 2cm }),
    header: context if counter(page).get() > (1,) {
      box(height: 2em, inset: (bottom:1em),
        stack(dir: ltr,
          text(fill: gray, titre),
          h(1fr),
          text(fill: gray, cours),
        ),
      )
    },
    footer: context
      if counter(page).get() > (0,) {
        stack(spacing: 0.7em,
          line(length: 100%),
          stack(dir: ltr,
            if auteurs_footer {
              stack(..auteurs.map(author => box(height: 1.6em, text(style: "italic")[#author.nom])))
            },
              h(3em),
            if auteurs_footer {
              stack(..auteurs.map(author => box(height: 1.6em, [#author.cip])))
            },
            h(1fr),
            counter(page).display(),
          ),
        )
      }
  )
  set text(font: "Noto Serif", size: 11pt, lang: "fr")
  set heading(numbering: "1.1")

  // Set run-in subheadings, starting at level 3.
  show heading: it => {
      set text(fill: rgb("#088c4c"))
      v(.4em)
      box(it)
  }

  show outline.entry.where(level: 1): set block(above: 1.2em)

  // Sets the equations properties

  set math.equation(numbering: "(1)")

  counter(page).update(0)

  // Title page
  stack(
    spacing: 1em,
    align(center, text([*Université de Sherbrooke*])),
    align(center, [*Faculté de génie*]),
    align(center, [*Département de génie informatique*]),
  )

  v(1fr)

  align(center, text(2em, weight: 700, titre))
  v(1em)
  stack(
    spacing: 1em,
    align(center, text(1.5em, cours)),
    align(center, text(1em, code))
  )

  v(1fr, weak: true)

  stack(
    spacing: .8em,
    align(center, [Par:]),
    ..auteurs.map(author => align(center, [#author.nom - #author.cip]))
  )

  v(3em)

  stack(
    spacing: .8em,
    align(center, [Présenté à:]),
    align(center, [L'équipe professorale]),
  )

  v(1fr, weak: true)

  align(center, text([Remis le #date]))

  // Table of contents.
  pagebreak()
  outline(depth: 3)

  v(3em)

  outline(
    title: [Table des figures],
    target: figure.where(kind: image),
  )


  // Main body.
  pagebreak()
  set par(justify: true)

  body
}

#let Annexe(breakPage : true, title : "Annexe", body,) = {

  if breakPage {
    pagebreak() // Optional: Start appendix on a new page
  }

  // Set the heading numbering specifically for the content within the Annexe
  // This will apply to any headings (e.g., '= ', '== ') placed inside the Annexe block.

  // You can explicitly add a top-level "Annexe" heading if desired.
  // If you want it to not be numbered, use `set heading(numbering: none)` just for this one.
  // For now, it will follow the `numbering` argument you pass in.

  [= #title <annexe>]

  counter(heading).update(0)
  set heading(numbering: "A.1.")

  // Render the content passed into the Annexe function
  body
}
