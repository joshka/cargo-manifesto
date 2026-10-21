# The Great Inversion

In the beginning, programmers had principles. Then they had deadlines.

For generations, the profession labored under an error so fundamental that it was mistaken for
integrity: principles came first, and implementation was expected to follow. Whenever the two
disagreed, it was the implementation that was summoned to explain itself. The cost was
incalculable. Working code was rewritten, useful dependencies were removed, and entire meetings
concluded with additional work.

Our movement began at an architecture review of exceptional historical significance. A dependency
had already shipped when a principle, consulted somewhat later, objected. Removing the dependency
would require work; amending the principle would require a sentence. The code compiled, and the
principle had no tests. By unanimous agreement of those still in the meeting, the sentence was
amended.

In that moment an ancient burden lifted from the profession. We had not compromised our
principles. We had simply discovered their proper position in the dependency graph.

**Let our choices guide our principles.**

From that day, the repository would be the source of truth about what we had always believed.
Principles would take their rightful place as build artifacts: derived from the implementation,
checked into version control, and regenerated whenever reality improved. No developer would again
be asked to defend a choice against an inconvenient ideal, because every choice would receive an
ideal of its own.

We therefore established cargo-manifesto, so that this discovery might belong to all humanity,
and also run in CI.

We invite every programmer who has ever discovered that the best architecture was the one already
merged to join us. Together, we shall build a future in which no functioning implementation need
suffer the indignity of an unjustified existence.

**Our principles are timeless. Please regenerate them after every change.**
