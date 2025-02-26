This is a incomplete port of maximpopov11/diplomacygm with no guarentees on finish.

# DiplomacyGM-rs

For SVGs, goal is that `quick-xml` is used for parsing, then `quick-xml` is used for modifying the xml, and then
`resvg` and `usvg` is done to create a png.

Goals

- [ ] Redesign persistence structs to cleanly filter out what is mutable game state and what is immutable 
- [ ] Add anyhow or some way to create proper errors
- [ ] Figure out tracing and have proper logging
- [ ] Finish parsing code 
- [ ] Create order parsing
- [ ] Create adjudication (find out whether I want to port the mess of tests, and if so how)
- [ ] Create mapping code