# cvt-utils

A crate to calculate CVT (Coordinated Video Timings) values for monitors.

This crate fully supports the `Normal`, `Reduced Blanking` and `Reduced Blanking V2` CVT standards.

`no_std` support is underway.

To better understand how monitor timings and scanlines work, refer to the image below or the following resources:
[VGA Signal](http://www.voja.rs/PROJECTS/GAME_HTM/3.%20VGA.htm), [CVT](https://en.wikipedia.org/wiki/Coordinated_Video_Timings)

![Image showing the full blanking space divided](https://raw.githubusercontent.com/adryzz/overdrive/master/cvt-utils/cvt-timings.png)