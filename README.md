# Albedo

Rust framework dedicated to real-time visualization.

## Disclaimer
---

🚧 Albedo is a work-in-progress and might be unstable, use it at your own risks 🚧

## Goals
---

* Lightweight
* Easy to use
* Fast
* Oriented for real-time visualization

Albedo **isn't** and will **never** be a game engine. It's designed to be a
rendering framework made for real-time visualization. It's possible to
use as the rendering module for a game, but that's not the use case why it
was designed.

## Features

### GPU Raytracing

The [albedo_rtx](./crates/albedo_rtx) exposes GPU software Raytracing. You can use this crate to perform Ray-Triangle intersections.

The [Albedo Pathtracer application](https://github.com/DavidPeicho/albedo) is one example of what you can achieve with the [albedo_rtx](./crates/albedo_rtx) crate:

![Pathtracing Example](https://github.com/DavidPeicho/albedo/raw/master/screenshots/initial_result.gif)
