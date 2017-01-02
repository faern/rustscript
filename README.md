# rustscript

Use Rust as a scripting language. Because RUST ALL THE THINGS etc. etc.

## Installation

Cargo can install rustscript directly from the git url like this:
```
cargo install --git https://github.com/faern/rustscript
```

Optionally install it globally:
```
sudo cp ~/.cargo/bin/rustscript /usr/local/bin/
```

## Writing a script

Just like a python or bash script start your file with a shebang telling your shell how to run your
script, like this: `#!/path/to/rustscript`.

`rustscript` will automatically wrap your code in a main method, so just start scripting directly
in the root of the file.

```rust
#!/usr/local/bin/rustscript

#[macro_use] extern crate shells;

use std::process;

let (code, stdout, _stderr) = sh!("ping -c 3 127.0.0.1");
println!("Ping output: {}", stdout);
process::exit(code);
```

Make the script executable and run it:

```
$ chmod +x ./my_rust_pinger.rsc
$ ./my_rust_pinger.rsc
```

### Importing external crates

Rustscript can pull in any crate from crates.io. As shown in the example above, just using
`extern crate foo` will pull in `foo` for you. However, you might want to be more specific than that

#### Importing specific versions of crates

Just giving the name of the crate, like `extern crate foo;` will always pull in the latest version
of that crate (`"*"` in Cargo.toml). You can specify a version requirement on the same format as
you would inside of `Cargo.toml` inside square brackets in the following way:

```rust
extern crate foo[*]; // Equivalent to just writing `extern crate foo;`
extern crate bar[0.1]; // Will add `bar = "0.1"` to Cargo.toml
extern crate baz[~1.1]; // Will add `baz = "~1.1"` to Cargo.toml
```

#### Importing crates with a different package name

When rustscript compiles your script it will by default use the same package name as the crate name.
This does not always work as some crates have different package name than crate name. To solve
this, rustscript allow you to specify a package name in the square brackets followed by a semicolon
and the desired version. If a package name is specified the version must be given and can't be left
out. Explicitly use `*` to get the latest version.

```rust
extern crate rustc_serialize[rustc-serialize;*]; // Will put `rustc-serialize = "*"` in Cargo.toml
```

#### Use external crates from local paths or git urls

This is not supported yet.

## Extra functionality

See the help output (`rustscript --help`) for more flags and extra functionality. For example, your
can make `rustscript` output the result of the script compilation even on success with:

`$ rustscript -v ./my_rust_pinger.rsc`

## Platform support

The idea is to support all major platforms, but because this is in an initial state only Linux
has been tested so far.

## Ideas for the future

* Automatic imports of large parts of stdlib for convenience
  * std::io::*;
  * std::fmt::*;
  * std::path::*;
* Allow importing crates from local paths and git urls
* Allow importing modules from other files (`mod foobar;`)
  * This is intentionally left out for now. If you need this then maybe you are not just writing a
    small script. Then maybe you should be using cargo as normal and write a regular crate.

## Internals

The first thing rustscript does is to calculate the hash of both the absolute path to your script
and the content of the script. It then looks for the folder `<user cache>/<path hash>`, where
`<user cache>` is the user specific cache directory given by `app_dirs`. That directory is called
the *script cache*. If the folder is missing or the hash of the script content does not match the
content of `<script cache>/script_hash` then a script build is initiated.

### Script build

Rustscript takes your script and creates a cargo crate out of it in the *script cache* directory.
It then builds that crate with the help of cargo. If the build fails the output of cargo will be
displayed and rustscript aborts.

### Script execution

If the cache did already contain the correct hash, or if the cargo build of the script succeeded,
then the built version of the script will be executed and all arguments given to the script will
be passed on to that subprocess.
