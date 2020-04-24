from wasmer import Instance
import os
from glob import glob

path = glob("*.wasm")[0]

wasm_bytes = open(path, 'rb').read()


def __wbindgen_json_parse(x,y):
  print(f"CALL __wbindgen_json_parse {x} {y}")

instance = Instance(
    wasm_bytes,
    {
        "env": {
            "__wbindgen_json_parse": __wbindgen_json_parse
        }
    }
)


