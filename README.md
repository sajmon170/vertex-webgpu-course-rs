# Vertex WebGPU course - lesson #1
This is the Rust version of the Vertex WebGPU course. Each branch represents a
different lesson. Be sure to check out the commit history - each commit adds a
single feature. Have fun!

## Programming environment setup
### Nix-based setup
This project provides a Nix flake. If you've got Nix installed with flake
support enabled you can just run `nix develop` to set up the whole environment.
If you don't have Nix, then you need to install the Rust toolchain manually.

### Manual setup
Follow the instructions on https://rustup.rs/ to install Cargo. A more detailed
explanation is provided in the
[Rust Book](https://doc.rust-lang.org/stable/book/ch01-01-installation.html).
Then install Git either through your package manager on Unix or through the
installer provided on https://git-scm.com/install/ on Windows.

Then open your terminal and clone this project using the `git` command:
```bash
git clone https://github.com/sajmon170/vertex-webgpu-course-rs.git
```

### Building and running the project
You can run this project by executing
```bash
cargo run
```
inside the cloned repository directory. This command will download the necessary
dependencies and build the project if required.
