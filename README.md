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

## API Improvement
---

* Use gfx-rs as a backend instead

* Instead of letting user manually manger memory / semd stuff, right a
CommandBuffer system that register addition / removal of mesh, modifications
of transform, etc...
Make this system avaiable in `Managed*` data-struct, like `ManagedRenderer`,
`ManagedIntersector`, etc...
