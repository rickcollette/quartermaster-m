#!/usr/bin/env python3
from pathlib import Path
import argparse

EOL = b"\x9b"

def to_atascii(data: bytes) -> bytes:
    return data.replace(b"\r\n", b"\n").replace(b"\r", b"\n").replace(b"\n", EOL)

def from_atascii(data: bytes) -> bytes:
    return data.replace(EOL, b"\n")

p = argparse.ArgumentParser()
p.add_argument("mode", choices=["to-atascii", "from-atascii"])
p.add_argument("input")
p.add_argument("output")
a = p.parse_args()
data = Path(a.input).read_bytes()
Path(a.output).write_bytes(to_atascii(data) if a.mode == "to-atascii" else from_atascii(data))
