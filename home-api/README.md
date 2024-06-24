# Home api

The home api is central unit for the home app. It is responsible for storing the environmental data from the home sensor and serving it on the website. The api is built using the Axum web framework and the website is built using the HTMX and Askama /HTML templates.

## Building

To build the project, you need to have the Cargo build system installed. You can install it by following the instructions on the [Rust website](https://www.rust-lang.org/tools/install).

To make the build successful, you need to have the following environment variables set:
- `API_SECRET` - the secret key for the API

After installing Cargo, you can build the project by running the following command:

```bash
cargo build --release
```

The binary will be located in the `target/release` directory.

## Running

To run the project, you can use the following command:

```bash
cargo run --release
```

The server will be running on `http://localhost:3000`.
