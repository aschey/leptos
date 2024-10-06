# Tachys

This is a soft fork of Leptos' Tachys that restores generic rendering.
Generic rendering was removed from the upstream repository due to compile time issues on large web projects (see [the PR](https://github.com/leptos-rs/leptos/pull/3015) for details).

This was forked for use in [Rooibos](https://github.com/aschey/rooibos), which is a framework for creating TUI applications.
My hope is that the widget trees for Rooibos are small enough so that the compile time issues described above won't be an issue.

I've also removed any web-specific code from this crate since it's no longer needed.
