# rustscript

Use Rust as a scripting language.

## TODO

* Submodules - The main script can import other files
  * Is this desired? If your program is multiple files it might be an application, not a script.
    Use normal Rust.
* Flags to rustscript to turn on/off
  * Compiler output
  * Force rebuild
* Automatic imports of large parts of stdlib for convenience
  * std::io::*;
  * std::fmt::*;
  * std::path::*;
