# rustscript

Use Rust as a scripting language. Because RUST ALL THE THINGS etc. etc.

## Installation

Cargo can install rustscript directly from the git url like this:
```
cargo install --git https://github.com/faern/rustscript rustscript
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

## Extra functionality

See the help output (`rustscript --help`) for more flags and extra functionality. For example, your
can make `rustscript` output the result of the script compilation even on success with:

`$ rustscript -v ./my_rust_pinger.rsc`

## Ideas for the future

* Automatic imports of large parts of stdlib for convenience
  * std::io::*;
  * std::fmt::*;
  * std::path::*;
