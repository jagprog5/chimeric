# Chimeric Engine

This is a design doc for chimeric engine - a cross platform runtime for 2D games with a focus
on safety and modding.

# Safe?

The program runs one or more bundles (folder of assets and scripts) together
in a sandbox. It can only take user input, draw to the screen, and save the game
state. The sandbox has configurations on the maximum resources it can use (cpu,
ram, vram, and disk when creating a save file).

At startup, each bundle can indicate a recommended amount of resources it would
like to use, and if this is larger than the existing configured maximum, then
the user will be prompted to provide an exemption. During runtime if these
limits are exceeded, a similar prompt will appear.

If an untrusted bundle is loaded, the worst it can do is:

 - mess up a save file
 - read assets from disk repeatedly (possibly wearing out hardware)
 - write assets to gpu repeatedly (possibly wearing out hardware)
 - saving to disk can only happen on explicit user action (ctrl-s or prompt)

## Zip bomb resistant

Some assets are compressed, like a jpg image. Only safe loading implementations
will be used, like [rust image](https://docs.rs/image/latest/image/io/type.Limits.html), which has configurable maximums (which will be exposed in the sandbox configs).

# Interface

A bundle interacts with the world through a provided interface:

 - __Textures Interface__,  built off of [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2) and sdl2-ttf, but not sdl2-image 
    - Request one of the following, and the sandbox will handle caching and memory management:
        - `image path` (all paths must be within the bundle's folder)
        - `font path, point size, text, option(wrapping width), color`
    - Once the mutable ref to the texture is obtained, it will have an API for
      drawing to, which window, with a src / dst, and optional rotation.
 - __Audio Interface__, (also rust-sdl2, sdl2-mixer)
    - Request a sound via path. Restricted to uncompressed formats (for now), as
      I can't find anything like rust-image which provides upper bounds on
      decompression, but if a upper limit can be deduced ahead of time then
      other formats can be added.
    - A set of N channels will be used internally. Each channel will only play one sound at a time.
    - When requesting to play a sound, a priority will be given (repr integer). If no channels are available, it will steal a channel from a lower priority sound, stopping it to take over, or won't play if none are available.
    - A set of effects includes the volume, panning, and whatever else is exposed by the underlying library.
    - To play a sound, `sound path, option(effects)` will be provided. This will give a handle which remains valid only while the sound is playing. The handle can be used to adjust the effects, and will do nothing (but will give feedback via API) if it isn't valid.
 - __Window Interface__  
    Multiple windows can be created, but it will be fail on platforms which
    don't support it (e.g. mobile). Window setup will be available on init (see
    below).
 - __Event Interface__  
    There will be a simple wrapper which allows for querying of events (mouse,
    keyboard, etc.). This will be a mutable reference, as it allows a bundle to
    consume / modify events before they reach a different bundle.
 - __Global State__  
    When the game's scripts are called by the engine, every bundle will be
    provided with the same global mutable json-like structure arg which persists
    over multiple frames and drives the overall state of the game. This is one
    of the facets for easy mod support - any bundle can edit data as desired.

    It will have a configurable maximum memory footprint, and the schema is
    entirely user defined and will be saved when creating a save file.

### Scripts

Code will be written in an embedded scripting language called [rune](https://github.com/rune-rs/rune).

 - has module support, allowing bundles to expose public api to other bundles
 - pass arbitrary json-like structures by mutable reference to a rune runtime.
   custom functions. does everything which is needed
 - can disable all vm functionality (network, fs, etc). restricted interaction
   with outside world to aforementioned interfaces
 - memory and runtime limits
 - in my opinion, better than rhai
    - seems simpler, while doing exactly what is needed
    - rune has [pattern
      matching](https://rune-rs.github.io/book/pattern_matching.html?highlight=match#pattern-matching)
      on enums, useful for matching sdl event type in rune code. rhai requires
      some [work arounds](https://rhai.rs/book/patterns/enums.html)
    - [faster?](https://github.com/khvzak/script-bench-rs)

At the top level, there will be a single definition of `loop` which is called
each frame.

All bundles' scripts will share the same symbol space, but modules are
recommended.

## Hot Reloading

If an asset is modified then it will be reloaded. Weather it be a image, font,
script, sound. Anything!

## Function Overriding

This is another facet of mod support.

A global hashmap associates a string to a rune function. Suppose there is an
entity with some member function "move". Instead of calling the move function
directly, a level of indirection can be used in which the function is looked up
from this map. This map will be made available to all bundles, so that entries
can be replaced if desired (instead of calling "move", call the modified "move",
as registered by a mod). Of note, it is up to the creator of a bundle to
"opt-in" to this functionality and use this indirection where they see fit.

### Init

This table must be populated in a deterministic and configurable order; some
mods should be loaded in before others. To facilitate this, a bundle's code can have a special comment line that is treated like a directive. It will contain the following information:

1. the fully qualified path of an init function that will be called at the beginning.
e.g. ["module", "inner_mod", ..., "my_setup_function"]  
that function will be given some args, and can do general setup (window size, etc),
and register any functions in the indirection map

2. it will indicate some ordering constraint. for example, the above function must be called before / after some other init function.
