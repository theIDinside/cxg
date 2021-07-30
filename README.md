# cxgledit
So what's this? Well this is cxgledit rewritten in rust, because I'd figured it would be more fun to write in Rust. And you'll always learn something
when you do something. Text editors are pretty small in scope, unless of course you want to write massively efficient insanely fast ones. The thing is though
just writing one in C, Rust, C++, D, by default will get you a lot of speed and efficiency. So the nice thing is you can worry about optimizing later.

So far, only some 60-70% of the same features as the C++ version, with some minor differences.


Key-bindings are not re-mappable right now, but obviously will be

- Select text
  - Shift + Cursor movement
  - Mouse click + drag
- Debug interface, show memory usage, frame time, fps, pid etc... more to come here
  - Ctrl + D
- New view & Buffer
  - Ctrl + N
- Close currently active view
  - Ctrl + W
- Cycle focused tab
  - Ctrl + Tab
- Debug dump a rust source file to active buffer
  - Ctrl + F1
- Go to end of file
  - Ctrl + End
- Go to beginning of file
  - Ctrl + Home
- Navigate cursor on word boundaries
  - Ctrl + Left/Right
- Show popup view (no other functionality than a normal view as of now)
  - Ctrl + P
- Open Input box (screen shot below)
  - Ctrl + shift + I
- Navigate text on source code block boundary
  - Shift + Alt Left/Right
- Navigate text on "word" boundary
  - Ctrl + Left/Right'
- Move Text View To another view's position
  - Ctrl + Mouse click & drag
  - Mouse click on title bar and drag

## Code quality
I have to be very clear and say that the quality of this code, how it's designed, is particularly awful. That mostly has to do with my inexperience
with writing applications that deal directly with OpenGL. Therefore most of the stuff becomes hack and slash (and I do mean, wild, hack and slash) to
get it working, and once I've done that, start thinking about architecturing the design. (one example of this hack-n-slash-then-design, is what I
did with DrawCommandList in [the polygon renderer](src/opengl/rectangle.rs))


### Todos (both complex & simple basic features)
- [ ] Syntax highlighting, using something like the regex crate which has been added to the [cargo configuration](Cargo.toml)
- [ ] Todo source code parser. Scan documents for todo comments and present them in some nice way
- [x] Buffer can now be hashed for comparison to saved contents, if the buffer is pristine / restored.
      etc.
- [ ] LINE WRAPPING. This. Must. Be. Done. Soon. Without it, the editor is bad.
- [x] Selecting text, with mouse and keyboard & rendering the selection properly.
- [ ] Other search / go to features (probably also using the regex crate)
- [ ] Symbol navigation. Like most things, I could start by using dependencies here, since the rust eco system is so powerful.
      One way of doing it, would be to do a really brute force approach and just scan the project, build a symbol database in an ad-hoc (and non-type safe way)
      and do it like that. No semantical analysis, nothing. Just eat_char(ch) until done, and figure out what are types, values, etc and use this to syntax highlight.
      Or, we can pull in parsers and lexers from other crates. We'll see. 


## Screenshots

Some look-and-feel showing
![gif rendering of simple use cases](docs/img/rendering.gif)

Empty file
![Start screen](docs/img/empty_file.png)

Editing while having 2 views open
![How regular editing looks right now](docs/img/editing.png)

Debug interface overlaid on the window
![How regular editing looks right now](docs/img/debug_interface.png)

Input box, emulating the functionality of all modern IDE's or editors, like VSCode, or Clion etc. The design
is bound to change, right now it's just about getting the functionality to work. Looks will come later.
Right now, there's no actual functionality in it. When typing in it, it produces files & paths in the workspace
folder, that contains the characters. The functionality isn't particularly hard. The UI is my absolute weakest side.

### Bugs
Figured out that the NFD library wasn't buggy. The massive spike in VMSize has nothing to do with actual *physical* allocated memory. So VMSize really only shows how much virtual memory is addressed. The real allocation is the Resident Set Size,
which accounts for memory that can be accessed without triggering a page fault interrupt (i.e swapping into memory the pages).
So RSS is a much better metric for resource/memory usage.

Keyboard command
![Input box for quick select of file browsing](docs/img/example.png)