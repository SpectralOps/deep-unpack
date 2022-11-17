# deep-unpack

Packing deep archives files from a root folder.

## Usage 
```toml
[dependencies]
deep-unpack = { version = "0.1.0" }
```

## Usage 
```rs

fn main() {

    fn main() {
         deep_unpack::DeepWalk::new()
        .folder("app/")
        .unpack_folder(format!("app/__extract__"))
        .unpack_level(4)
        .extract()?;
    }
}
```

[All the examples here](./unpack/examples/README.md)


## Thanks
To all [Contributors](https://github.com/spectralOps/deep-unpack/graphs/contributors) - you make this happen, thanks!


## Copyright
Copyright (c) 2022 [@kaplanelad](https://github.com/kaplanelad). See [LICENSE](LICENSE) for further details.