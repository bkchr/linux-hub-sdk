# Geeny Hub SDK

## Introduction

The Geeny Hub SDK provides an abstraction over APIs and communication
interfaces necessary to connect physical or virtual devices to the Geeny
cloud. The Hub SDK can be used in one of two ways: as a Rust library crate, or as a
a standalone binary that can be used by applications written in other
languages by providing an interprocess communication interface.

For developers with an existing Hub Device, such as a Smart Home Gateway,
the standalone Geeny Hub Service may be used to provide a single local interface
to communicate with Geeny Cloud Services to enable device provisioning, sending
messages to the cloud, and receiving messages from the cloud. The Geeny Hub Service
may be installed as a package, or included as part of a firmware generation process
such as Buildroot or Yocto. When used as a service, no knowledge of Rust developement
is necessary.

For developers building a new Hub Device, the Geeny Hub SDK may be used as a library
(or crate), and can be tightly integrated into a Rust-based application. This allows
developers to interact with the Geeny Cloud through an idiomatic Rust library interface,
rather than having to implement REST, MQTT, and other communication protocols.

## Components

### Usage - As a Library Crate

```rust,no_run
extern crate hub_sdk;

use hub_sdk::{HubSDK, HubSDKConfig};

fn main() {
    let sdk_cfg = HubSDKConfig::default();

    // Begin running the SDK. The hub_sdk handle may be used to interact with
    // the sdk. This handle may be cloned and given to multiple consumers
    let hub_sdk = HubSDK::new(sdk_cfg);

    let msgs = hub_sdk.receive_messages("ABC123")
        .expect("No known device with that serial number");

    println!("Messages: {:?}", msgs);
}
```

#### Documentation

Full library documentation may be found on [docs.rs](https://docs.rs/hub-sdk), or may be generated
from this repository using `cargo doc --open`.

### Usage - As a standalone service

```bash
# Create a valid config file for this service
cp ./geeny_hub_service.mvdb.json.example ./geeny_hub_service.mvdb.json

# Run the service, serving a REST IPC on localhost:9000
cargo run --release --bin hub-service
```

#### Documentation

For more information regarding the REST IPC interface, please see
[this Swagger API specification](./docs/rest-ipc/swagger.json)
for more information.

## Requirements

Currently, the Geeny Hub SDK requires a nightly build of Rust.

## Installation & Configuration

### As a library

In your `Cargo.toml`, add the following lines:

```toml
[dependencies]
hub-sdk = "0.3"
```

In your main project file (likely `lib.rs` or `main.rs`), add the following line:

```rust,ignore
extern crate hub_sdk;
```

### As a service

```bash
# Create a valid config file for this service
cp ./geeny_hub_service.mvdb.json.example ./geeny_hub_service.mvdb.json

# Run the service, serving a REST IPC on localhost:9000
cargo run --release --bin hub-service
```

## Testing

Unit tests may be run with `cargo test`.

## License

Copyright (C) 2017 Telefónica Germany Next GmbH, Charlottenstrasse 4, 10969 Berlin.

This project is licensed under the terms of the [Mozilla Public License Version 2.0](LICENSE.md).

Contact: devsupport@geeny.io

### Third Party Components

This crate makes use of the following third party components with the following licenses:

| License | Count | Dependencies |
| :------ | :---- | :----------- |
| Apache-2.0 | 2 | openssl, thread-id |
| Apache-2.0/MIT | 76 | antidote, backtrace, backtrace-sys, base64, bitflags, bitflags, cfg-if, coco, cookie, core-foundation, core-foundation-sys, custom_derive, dtoa, either, env_logger, error-chain, foreign-types, futures, gcc, httparse, hyper-native-tls, idna, isatty, itoa, lazy_static, libc, log, mqtt-protocol, native-tls, num-traits, num_cpus, ordermap, pear, pear_codegen, percent-encoding, pkg-config, quick-error, quote, rand, rayon, rayon-core, regex, regex, regex-syntax, regex-syntax, reqwest, rocket, rocket_codegen, rocket_contrib, rustc-demangle, scopeguard, security-framework, security-framework-sys, serde, serde_derive, serde_derive_internals, serde_json, serde_urlencoded, state, syn, synom, tempdir, thread_local, thread_local, threadpool, time, toml, traitobject, unicode-bidi, unicode-normalization, unicode-xid, unreachable, url, uuid, vcpkg, yansi |
| BSD-3-Clause | 3 | adler32, magenta, magenta-sys |
| MIT | 21 | advapi32-sys, conv, crypt32-sys, dbghelp-sys, hyper, kernel32-sys, language-tags, libflate, matches, mime, mvdb, openssl-sys, redox_syscall, schannel, secur32-sys, typeable, unicase, version_check, void, winapi, winapi-build |
| MIT OR Apache-2.0 | 1 | safemem |
| MIT/Unlicense | 9 | aho-corasick, aho-corasick, byteorder, byteorder, memchr, memchr, rumqtt, utf8-ranges, utf8-ranges |
| MPL-2.0 | 2 | geeny-api, smallvec |
| Other`*` | 2 | ring, untrusted |

`*` Please see [ring's license](https://github.com/briansmith/ring/blob/master/LICENSE) for more
information regarding `ring` and `untrusted`'s licensing.