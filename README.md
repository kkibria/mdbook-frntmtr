# mdbook preprocessor to support front matters

```text
---
title: Software Development Cookbook
---

# {{ page.title }}

This is mostly a collection of **gists** I noted from my own experience
and many others that I collected over the years and still collecting.
```

The handlebar styled variables will be replaced from values defined in front matter.



------ note for debugging.

Nice — VS Code + CodeLLDB is perfect for this.

## 1) Build config you want

You want to run the **debug** binary from the **mdBook project folder** (so `book.toml` resolves correctly).

So in VS Code, open the `mdbook-frntmtr` repo, and add a launch config that sets `cwd` to your book directory.

## 2) Create `.vscode/launch.json` in `mdbook-frntmtr`

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "mdbook-frntmtr: serve (debug)",
      "type": "lldb",
      "request": "launch",
      "cargo": {
        "args": ["build"],
        "filter": {
          "name": "mdbook-frntmtr",
          "kind": "bin"
        }
      },
      "args": ["serve", "--", "--open"],
      "cwd": "/Users/user/dev/quickaadon_book",
      "env": {
        "RUST_BACKTRACE": "1"
      },
      "console": "integratedTerminal"
    }
  ]
}
```

### If you want to force mdbook path (optional but useful)

Add:

```json
"env": {
  "RUST_BACKTRACE": "1",
  "MDBOOK": "/Users/user/.cargo/bin/mdbook"
}
```

(Replace that path with whatever `which mdbook` prints.)

## 3) Where to put breakpoints

Set breakpoints in:

* `run_serve(...)` (main.rs)
* right after `config::load(...)`
* right before `Command::new(...).current_dir(...).status()`

Then inspect:

* `sc.book_root` (does it exist?)
* `sc.template_path` (does it exist?)
* the resolved `mdbook` executable path (if you added `find_mdbook()`)

## 4) Most likely root cause to confirm quickly

Even if `mdbook` exists, **`current_dir(&sc.book_root)` being wrong/nonexistent** will produce the same `os error 2`.

So in the debugger, as soon as `sc` is loaded:

* check `sc.book_root`
* check `std::fs::metadata(&sc.book_root)` (or just `exists()`)

If it’s wrong, fix is in `config::load()` path resolution (usually joining/parent logic).

If you hit a breakpoint and want me to sanity-check what you’re seeing, paste the values of:

* `sc.book_root`
* `sc.src_dir`
* `sc.template_path`
* and the mdbook path you’re spawning

