# ytcmt

This is a tool to pull comments from youtube videos

## Build & Run

```sh
cargo build --release
cargo run --release
```

> [!NOTE]
> This tool is under development.
>
> **usage example**
>
> ```sh
> cargo run --release -- -m 6 "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=RDdQw4w9WgXcQ&start_radio=1"
> ```
>
> Curently, the output will be created in `/comments.json`
