# The Great Inversion

In the beginning, programmers had principles.

Then they had deadlines.

For generations, the profession labored under an error so fundamental that it was mistaken for
integrity: principles were established first, and implementation was expected to follow. Whenever
the two disagreed, the implementation was summoned to explain itself.

The cost was incalculable. Working code was rewritten. Useful dependencies were removed. Entire
meetings concluded with additional work.

Our movement began at an architecture review of exceptional historical significance. A dependency
had already shipped. A principle, consulted somewhat later, objected.

Removing the dependency would require work. Amending the principle would require a sentence.

The code compiled. The principle had no tests.

By unanimous agreement of those still in the meeting, the sentence was amended.

And in that moment, an ancient burden lifted from the profession. We had not compromised our
principles. We had discovered their proper position in the dependency graph.

**Let our choices guide our principles.**

The repository would henceforth be the source of truth about what we had always believed. Principles
would assume their rightful place as build artifacts: derived from implementation, checked into
version control, and regenerated whenever reality improved.

A small dependency graph would demonstrate independence. A large one would demonstrate solidarity.
Moving between them would confirm that our principles had never changed.

Exact versions would express prudence. Compatible ranges would express trust. A project employing
both would demonstrate the judgment required to know which virtue a decision had already exercised.

No longer would developers be asked to defend a choice against an inconvenient ideal. Each choice
would receive an ideal of its own.

We therefore established cargo-manifesto, that this discovery might belong to all humanity, and also
run in CI.

Whenever the code and the manifesto disagree, the manifesto shall be corrected. The command is
`--fix`.

Whenever a decision reverses, the historical record shall explain how both positions arose from the
same enduring conviction. The command is `--amend`.

We invite every programmer who has ever discovered that the best architecture was the one already
merged to join us.

Together, we shall build a future in which no functioning implementation need suffer the indignity
of an unjustified existence.

**Our principles are timeless. Please regenerate them after every change.**
