# ytcmt

This is a tool to pull comments from youtube videos

## Build & Run

```sh
cargo build --release
cargo run --release
```

> [!NOTE]
> This tool is under development. If you want to pull comments from a specific video, change the `VID_ID` env variable
>
> **example** `https://www.youtube.com/watch?v=dQw4w9WgXcQ&start_radio=1`
>
> ```sh
> VID_ID=dQw4w9WgXcQ cargo run --release
> ```
