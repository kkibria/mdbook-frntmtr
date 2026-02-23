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



Note:
> mdbook-frntmtr works as a preprocessor (supports/run)
> mdbook-frntmtr serve is an optional convenience wrapper around mdbook serve with a watcher

