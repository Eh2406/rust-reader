This is a light weight Windows screen reader intended for persons that need some selectable text read and do not need the entire UI read. It is written in Rust.

The primary function is the `Read` hotkey by defalt `Ctr-/` witch:
1. Save the contents of the clipbored
2. Types `Ctr-c`
3. Start reading the new contents of the clipbored
4. Put the clipbored back to the contents it saved in step 1.

----
Upgrade and Installation (after rust is installed)
----
1. Get the latest Rust release. On the command line type `rustup update`.
2. Replace the source code with the code from github.
3. Close the reader
3. Build reader by typing on the command line `cargo build --release`
4. Reopen reader.

----
Prehistory
----

[Chaim](https://github.com/toChaim) and [I](https://github.com/Eh2406) got frustrated with the screen readers we were using. We decided that it didn't take much to be more useful than the available options. So we worked together and made a very functional alternative.

It had:
- Multi window support; you could have many different documents open in separate reader windows.
- A pronunciation editer; if a word was mispronounced you could use regex to substitute a phonetic spelling.
- A ticker tape; you could watch words being read run across the top of your screen.
- Skip forward and back by time, sentence, or paragraph.
- Full persistence; on launching the program it would have restored all the windows you had open.
- A progress bar complete with percent complete and time left.

Unfortunately, we had stretched out our programing language AutoIt3 far past its limits. Later, when I was learning rust, I went back to fix sum bugs, but discovered it was too unwieldy and so started this rewrite in rust.