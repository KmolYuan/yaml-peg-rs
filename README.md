# yaml-peg

[![dependency status](https://deps.rs/repo/github/KmolYuan/yaml-peg-rs/status.svg)](https://deps.rs/crate/yaml-peg/)
[![documentation](https://docs.rs/yaml-peg/badge.svg)](https://docs.rs/yaml-peg)

A [YAML 1.2](https://yaml.org/spec/1.2) parser using a greedy parsing algorithm with PEG atoms. Support anchors, directives, positions, tags, serde, and no-std.

Inspired from [`yaml-rust`](https://github.com/chyh1990/yaml-rust) and [`serde-yaml`](https://github.com/dtolnay/serde-yaml).

This parser is not ensure about YAML spec but almost functions are well-implemented. The buffer reader has also not yet been implemented, but the chunks can be read by the sub-parsers.

The anchors can be visited by the loader after parsing.

```rust
use yaml_peg::{parse, node};

let doc = "
---
name: Bob
married: true
age: 46
";
let root = parse(doc).unwrap();
assert_eq!(root, vec![node!({
    "name" => "Bob",
    "married" => true,
    "age" => 46,
})]);
```

See the API doc for more information.

## Features

+ Support no standard library `#![no_std]`.
+ Different data holder `Rc` / `Arc` provides parallel visiting and less copy cost.
+ Provide document positions and tags on the nodes.
+ Support anchors / alias.
  + Direct mode: Embed the alias directly.
  + Cyclic mode: Keep the alias placeholder, for cyclic data.
+ Support YAML directives `YAML` and `TAG`.

  **WARNING: `%YAML 1.1` will still be treated as 1.2.**

  ```yaml
  %YAML 1.2
  %TAG !x! tag:my.prefix:
  ---
  ```
+ Support [`serde`](https://github.com/serde-rs/serde) to help you serialize and deserialize a specific type. (as well as the non-cyclic anchors)
  ```rust
  use serde::Deserialize;
  use yaml_peg::serde::from_str;

  #[derive(Deserialize)]
  struct Member {
     name: String,
     married: bool,
     age: u8,
  }

  let doc = "
  ---
  name: Bob
  married: true
  age: 46
  ";
  // Return Vec<Member>, use `.remove(0)` to get the first one
  let officer = from_str::<Member>(doc).unwrap().remove(0);
  assert_eq!("Bob", officer.name);
  assert!(officer.married);
  assert_eq!(46, officer.age);
  ```
