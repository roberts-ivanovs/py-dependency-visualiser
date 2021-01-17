# Mermaid import visualiser for Python

Generate a dependency/import graph for a python module. Useful when re-writing existing Python libraries to other languages.

## Build

```sh
cargo build --release
cp ./target/release/import_visualiser ./import_visualiser
```

Example use-case

```sh
import_visualiser /mnt/BaseStorage/doodles/rust/shit/python-zeep/src/zeep --name-filter zeep
```

This will look at all the python files in the given directory and construct a mapping of the elements for only the imports that are starting from the "zeep" base root:

- `from zeep.xsd import elem`  -- will get included
- `from datetime import datetime`  -- will not get included

## Output

Output will be generated in the execution directory file `output.md`.
The generated item will be a [mermaid](https://mermaid-js.github.io/mermaid/#/flowchart?id=graph) graph.
