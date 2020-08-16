<center>
    <h1><code>bluenoise-rs</code></h1>
    <img src="https://img.shields.io/crates/v/bluenoise-rs?style=flat-square"/>
    <img src="https://img.shields.io/crates/l/bluenoise-rs?style=flat-square"/>
</center>

`bluenoise` provides an implementation of poisson disk sampling
in two dimensions, with `glam` as the underlying maths library.
It aims to be fast, well documented and easy to use, taking
advantage of [a few optimisations](http://extremelearning.com.au/an-improved-version-of-bridsons-algorithm-n-for-poisson-disc-sampling/)
to dramatically speed up compute speed.
