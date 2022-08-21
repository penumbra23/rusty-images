# Rusty Images

Single API image transformation service written in Rust.

## Running

Simply clone the repo and run inside the root directory:
```
cargo run
```

or run with Docker

```
docker run -it -p 3000:3000 penumbra23/rusty-images
```

## API

Run `cargo run -- --help` to inspect the arguments that can be supplied via CLI.

All endpoints receive the image in a `multipart/form-data` body with the entry name `file`.

#### `POST /stats`

Returns image information like size (in bytes), width, height and format:
```json
{
    "size": 161985,
    "width": 985,
    "height": 2048,
    "format": "image/jpeg"
}
```

#### `POST /resize/{width: u32}/{height: u32}`

Resizes the image. Possible query params:
- `keep_aspect=true` - if the image aspect ratio should be retained; in that case the highest dimension is picked
- `output_format` - convert to another format (possible values: `png`, `jpeg` and `gif`)
- `filter_type` - filter type to be used while rescaling (possible values: `nearest` and `gaussian`)

#### `POST /blur/{strength: f32}`

Blurs the image using the Gaussian filter with the given strength factor. Possible query params:
- `output_format` - convert to another format (possible values: `png`, `jpeg` and `gif`)

#### `POST /rotate/{angle: u32}`

Rotates the image using one of the possible values for the angle = [90, 180, 270]. Possible query params:
- `output_format` - convert to another format (possible values: `png`, `jpeg` and `gif`)

## Contribute

For any bugs, feature requests or improvements, open up an issue and/or submit a PR. More than happy to see contributors.
One of the major things this service could use is an async API, where the operations are queued and executed in an async fashion.

## License
MIT
