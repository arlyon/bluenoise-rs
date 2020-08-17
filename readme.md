<center>
    <h1><code>bluenoise-rs</code></h1>
    <img src="https://img.shields.io/crates/v/bluenoise?style=flat-square"/>
    <img src="https://img.shields.io/crates/l/bluenoise?style=flat-square"/>
</center>

`bluenoise` provides an implementation of poisson disk sampling
in two dimensions, with `glam` as the underlying maths library.
It aims to be fast, well documented and easy to use, taking
advantage of [a few optimisations](http://extremelearning.com.au/an-improved-version-of-bridsons-algorithm-n-for-poisson-disc-sampling/)
to dramatically speed up compute speed.

## Get Started

To get started, if you have [`cargo-edit`](https://github.com/killercup/cargo-edit), simply run:

```
cargo add bluenoise
```

Otherwise, add `bluenoise` to your `Cargo.toml`.

```
[dependencies]
bluenoise = "0.1"
```
