# References and licensing policy

SyllaslopOS is MIT-licensed (copyright kamensuper). Every file in this
repository must be compatible with that license.

## Slopix (predecessor)

- Source: https://github.com/kamensuper/slopix (studied as a read-only clone outside this repo).
- License: the first-party code is MIT (`LICENSE-FIRST-PARTY-MIT`). The
  combined kernel is distributed under GPLv3 because it links DoomGeneric
  and other third-party code (`THIRD-PARTY.md`).
- Use: we study its design and its mistakes. First-party Slopix code may be
  adapted under MIT, but only after we understand its assumptions, and the
  commit must say it was adapted. Never take code from Slopix's
  GPL/third-party parts (`third_party/`, `vendor/`, the Doom port).

## Syllable Desktop / AtheOS

- The main design reference for the overall architecture (kernel/server
  split, application server, messaging, the native GUI toolkit) and for
  where the desktop goes in the future.
- License: mostly GPL (kernel and most system components). Some libraries
  are LGPL.
- Use: **reimplement, never copy.** Read the docs and source to understand
  the concepts and interfaces, then write our own implementation. Do not
  paste code, transliterate it line by line, or reproduce non-trivial
  structures verbatim.

## Policy summary

1. Writing it ourselves is the default.
2. Outside code needs a license check first. MIT/BSD/ISC is acceptable with
   attribution. GPL/LGPL is not, unless the user decides to change the
   project license.
3. Record any adapted code here and in its commit message.
