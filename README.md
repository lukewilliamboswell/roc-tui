
## Terminal UI Platform for Roc

This is a tui platform for Roc which essentially just wraps tui-rs.

My goal is to learn more about Roc platform dev, and explore ideas for the Roc editor, and how to do [Action-State in Roc](https://docs.google.com/document/d/16qY4NGVOHu8mvInVD-ddTajZYSsFvFBvQON_hmyHGfo/edit#).

**I welcome any feedback or assistance!**

## Things I'm working on, or thinking about
- [x] Block widget
- [x] Paragraph widget
- [x] Layout widget
- [x] Styling
- [x] Scrolling for paragraphs
- [x] Optional Records for better API
- [ ] Cusor on paragraphs to make input boxes
- [ ] Fix support newline characters
- [ ] List widget with selection
- [ ] Write more tests
- [ ] Support mouse input
- [ ] Better error handling, don't mess up terminal if Roc panics somehow 
- [ ] Support more widgets blocked on [#4554](https://github.com/roc-lang/roc/issues/4554)

## Simplified process to add functionality
1. Review the [tui-rs docs](https://docs.rs/tui/0.19.0/tui/) and [examples](https://github.com/fdehau/tui-rs/tree/master/examples) to understand the behaviour.
2. Add feature to the [Platform API](./platform/main.roc) `*.roc` files
3. Generate `platform/glue.rs` with `roc glue platform/main.roc platform/src/glue.rs`
4. Use `cargo build` from the platform folder to fix any errors
5. Wire functionality into the platform host Rust code 
6. Update `hello.roc` example and iterate 