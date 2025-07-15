# A Rust + WebAssembly Hex Editor

<div align="center">
  <img src="https://rustacean.net/assets/rustacean-flat-happy.svg" width="150" alt="A happy Ferris the Crab">
</div>

<p align="center">
  <strong>A surprisingly powerful hex editor that runs entirely in your web browser, built with Rust and WebAssembly.</strong>
</p>

<p align="center">
  <img alt="License" src="https://img.shields.io/badge/Licence-GPLv3-blue.svg">
  <img alt="Language" src="https://img.shields.io/badge/language-Rust-orange.svg">
  <img alt="Built with" src="https://img.shields.io/badge/built%20with-WebAssembly-purple.svg">
</p>

---

Right then! So, what happens when you get a bit hyper-focused on Rust, WebAssembly, and the idea of doing things in a web browser that maybe, *just maybe*, belong in a desktop app? You get this.

This is the imaginitively named **HexEditor**, a tool for peeking into the binary guts of files, right from the comfort of a browser tab. No installation, no fuss, just pure, unadulterated byte-level fun.

---

## Why, though?

Honestly? Because it seemed like a fun challenge. I wanted to see if I could build a tool that felt snappy and responsive, like a native application, but using the modern web stack. It was also a brilliant excuse to dive deeper into some fantastic technologies:

* **Rust:** For its safety, performance, and frankly, its rather elegant way of making you think properly about your code.
* **WebAssembly (WASM):** The magic that lets us run that compiled Rust code in the browser at near-native speeds. It's still a bit like witchcraft to me, and I love it.
* **The Yew Framework:** A Rust framework for building web apps, inspired by React. It lets you write your front-end entirely in Rust, which is just brilliant.
* **Virtualization:** The real secret sauce. I got a bit carried away and wanted to open a massive file... which, predictably, crashed the browser. That led me down the rabbit hole of only rendering what you can see, which means it can now handle huge files without breaking a sweat.

It's a testament to how powerful the modern web platform has become.

---

## Features

* **File Loading:** Chuck any file at it and see its contents in the classic hex/ASCII layout.
* **Blazingly Fast Virtualized Scrolling:** Handles massive files (tested up to several megabytes) with a silky-smooth scroll. It only renders the visible parts of the file, so your browser won't throw a wobbly.
* **Edit on the Fly:** Click a byte, change its value, and click away. The changes are saved to the application's memory.
* **Real-time ASCII View:** As you change a hex value (e.g., from `41` to `42`), the ASCII representation on the right-hand side updates automatically (from `A` to `B`). It's the little things!
* **Save Your Changes:** Once you're done tinkering, you can save the modified file back to your computer.
* **Dark Mode By Default:** Because we're not monsters. It's styled with Bootstrap's dark theme for maximum eye comfort during those late-night reverse-engineering sessions.

---

## Getting Started

Fancy running this on your own machine? Right then, let's get you sorted.

### Prerequisites

You'll need the Rust toolchain and Trunk, which is a fantastic bundler for Rust WASM applications.

1.  **Install Rust:** If you haven't already, get it from [rustup.rs](https://rustup.rs/).
2.  **Install Trunk:** Fire up your terminal and run:
    ```bash
    cargo install trunk
    ```

### Running the Project

1.  **Clone this repository:**
    ```bash
    git clone https://github.com/andydixon/hexeditor
    cd hexeditor
    ```
2.  **Serve it up!**
    Just run this one simple command:
    ```bash
    trunk serve --open
    ```
    Trunk will compile the Rust code to WASM, bundle everything together, start a local server, and automatically open the application in your default web browser. Bob's your uncle!

---

## ## A Look Under the Bonnet (How it Works)

The most interesting bit of this project is the virtualization. When you first try to render a massive file by creating an `<input>` for every single byte, the browser's DOM (Document Object Model) gets absolutely flooded. It has to manage hundreds of thousands, or even millions, of individual elements, and it just gives up.

The solution is to trick the browser.
1.  We create a main scrollable container with a fixed height.
2.  Inside it, we place another `div` that has the *total height* the table *would* have if all the rows were rendered. This gives us a correctly-sized scrollbar.
3.  We then listen for scroll events. Based on how far down the user has scrolled, we calculate which small slice of rows should actually be visible.
4.  Finally, we render only that small slice (plus a few extra rows above and below for smoothness) and use a CSS `transform` to position it exactly where it should be in the viewport.

The result? The browser is only ever managing a few dozen table rows at any given time, but you get a perfectly functional scrollbar that makes it feel like the whole file is there. Clever, eh?

---

## ## Future Ideas & Brain Dump

Right, so my brain didn't stop firing off ideas just because the main thing works. Here's a totally unordered list of things that would be absolutely brilliant to add, in no particular order whatsoever.

* [ ] **Themes!** A light theme, a Solarized theme, maybe a horrible 80s green-screen theme for a laugh.
* [ ] **Search Functionality:** The ability to search for specific hex sequences or ASCII strings. Crucial for any real work.
* [ ] **Data Inspector:** Highlight a selection of bytes and have a little pop-up that interprets them as different data types (a 32-bit integer, a float, a little-endian vs big-endian value, etc.).
* [ ] **Drag and Drop:** Just yeet a file onto the page to open it. Much cooler than the standard file input.
* [ ] **Undo/Redo:** Because I've already made a mess of a file by mistake. `Ctrl+Z` is muscle memory.
* [ ] **Diffing:** Load two files and highlight the differences. Now that would be a proper party piece.

---

## ## Licence

This project is licensed under the **GNU General Public License v3.0**.

This means you are free to use, share, and modify the software, but if you distribute your own modified version, you must also make the source code available under the same licence. See the `LICENSE` file for the full text.
