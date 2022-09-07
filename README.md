# Flutter_embedder

In a nutshell, this is a project about integrating Flutter with wgpu.

## Flutter

[Flutter](https://flutter.dev/) is a UI framework written by Google for mobile platforms that supports running on Android, iOS, macOS, Windows, Linux, Fuchsia and Web. It tries to get unified visualization and behavior of its UI by ignoring all of the built-in native UI and doing everything itself, with only a thin interface layer to the OS necessary.

For this, it can use a range of rendering APIs, namely OpenGL, Metal, Vulkan and plain software rendering.

On the application developer side, it uses Dart, a language developed by Google that can be compiled to machine code and JavaScript. The web variant uses the latter, while all other platforms use the former.

Flutter is based on defining the UI in a declarative style similar to React.

## Rust

[Rust](https://www.rust-lang.org/) is a systems language that emphasises reliability. For this project, one of the most important aspects is that it also supports a wide variety of platforms, including all mentioned above for Flutter.

## wgpu

There is a rendering library for Rust called [wgpu](https://wgpu.rs/) that abstracts away native rendering APIs into a common API that's easier to use. Based on this, many rendering frameworks like [bevy](https://bevyengine.org/) have been developed.

## Goal

The goal behind this project is to allow rendering frameworks like bevy to use a UI written in Flutter. The background is that all Rust-based UI frameworks are quite limited in their feature set. Google has invested a ton of time into writing Flutter, which can be leveraged for this.

The special thing about Flutter is that it has full support for using it as a library to integrate it into any kind of application. It supplies a C-based embedder API (so it's easy to interface with Rust) that allows detailed control over the rendering process via a compositor. This even allows interleaving Flutter UI with natively-rendered widgets (called platform widgets in Flutter).

Flutter and wgpu have two intersections for rendering APIs, which are Vulkan and Metal. Unfortunately, the official shells (these are the applications using the embedder) for native Flutter applications all use OpenGL, with the exception of iOS/macOS, which use Metal. Vulkan is only used on Fuchsia.

One important rendering API missing from Flutter is DirectX12, which is the only preinstalled one on Windows. The official Windows shell alleviates this by using [ANGLE](https://github.com/google/angle), which is an OpenGL implementation that can translate the calls to DirectX11. This is not a direction this project wants to take. On Windows, Vulkan can be installed by endusers by installing the graphics drivers for their system.

So, the goal of this project is to use the Vulkan rendering backend of Flutter in a wgpu context in order to integrate them both into a single window.

Flutter shells also have to supply all of the interface with the native operating system. In the official shells, this is done manually with a lot of code, and all of them are completely separate projects. This is a lot of work and not feasible here.

In the Rust world, there is [winit](https://github.com/rust-windowing/winit), which is an abstraction over various platforms to provide a single API for creating windows and receiving events (like mouse and keyboard input). This mostly matches the needs of the Flutter embedder.

### End Game

Once all of this works, the plan is to create a library crate and publish on crates.io. If you want to create a wgpu-application with Flutter, you have to create a regular Rust binary project and add the crate as normal. Then in your code, you somewhere have to initialize the library and run it. This is also where you register your own Platform Widgets (which might use bevy or whatever you like).

This is the same structure as used for the official shells.

Note that the Flutter engine is built as a shared library, so it needs to be shipped along with your own binary and all of the Flutter resources.

The web works completely differently there (since the engine is compiled to JavaScript, and there's HTML to bootstrap everything), so it's a non-goal for the project. However, there it's quite easy to use the official shell, because it allows defining a canvas as a platform widget, which then can be used for wgpu without any special preparations.

## Current State

Everything is highly experimental. This project is far from being usable for real applications!

- Opening the window and initializing the Flutter runtime works.
- Rendering the first layer of the Flutter UI works.
- Platform Views are missing.
- Rendering multiple layers is not implemented correctly, because there is no blending (it's just bitwise copying the texture).
- Resizing windows is buggy (buffer sizes are out of sync)
- Mouse input works
- Changing the mouse cursor works
- Keyboard input is halfway there.
  - There are three different APIs in Flutter for this: keyevent, keydata, and textinput. Keydata is optional, the other two are necessary.
  - keyevent and textinput are implemented
  - They are implemented using an experimental winit API from a pull request, because the stable API does not supply the information necessary.
  - keydata is problematic, because it requires to supply the keyboard events in a specific platform-specific format, which we don't have.
  - textinput is a very complex API, because all of the complexity of handling text is offloaded to the shell. IME support is missing, as is autocomplete and dictionary support.
- Only Linux is working in some aspects. The main reason is that the new winit API for keyboard handling hasn't been implemented for Windows yet. Also, there is no support for Metal right now for iOS/macOS.
- Mobile is not a focus at the moment, but might come later.

## Accessibility

This is always a big topic for UI frameworks. Flutter has full support for accessibility, including screenreaders. However, as expected this relies on the shell to provide the operating system interoperation.

Accessibility APIs differ widely between different operating systems, so this is hard to achieve with a project like this, where having a single codebase for all systems is a primary goal.

However, luckily there is a project called [AccessKit](https://github.com/AccessKit/accesskit) that aims to provide a way to add accessibility to winit as a cross-platform solution, offloading all platform-specific code to that crate. It is still in its early stages and the longevity is unclear, but _if_ it succeeds in providing what it aims to do, it is a prime candidate to solve the issue for this project.

## License

This project is licensed under the Apache 2.0 license. See [LICENSE](./LICENSE) for details.
