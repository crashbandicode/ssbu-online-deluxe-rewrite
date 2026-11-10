#!/usr/bin/env python3
"""Read or rewrite the account nicknames in an Eden profiles.dat (acc ProfileDataRaw: 0x10 bytes, then eight
0xC8-byte users: uuid 16, uuid2 16, timestamp 8, nickname 32 bytes UTF-8 zero-padded, extra data 0x80).

  nickname.py show <profiles.dat>
  nickname.py set <profiles.dat> <out> <nickname>   every valid user gets <nickname>; nothing else changes
"""
import hashlib
import sys

HEADER, USER, USERS, NAME_AT, NAME_LEN = 0x10, 0xC8, 8, 0x28, 0x20


def users(data):
    for i in range(USERS):
        base = HEADER + i * USER
        uuid = data[base:base + 16]
        if any(uuid):
            name = data[base + NAME_AT:base + NAME_AT + NAME_LEN].split(b"\0", 1)[0].decode("utf-8", "replace")
            yield i, base, uuid.hex(), name


def main(argv):
    data = bytearray(open(argv[2], "rb").read())
    if len(data) < HEADER + USERS * USER:
        raise SystemExit(f"{argv[2]}: {len(data)} bytes, expected at least {HEADER + USERS * USER}")
    if argv[1] == "show":
        for i, _, uuid, name in users(data):
            print(f"user {i} uuid {uuid} nickname {name!r}")
        return 0
    name = argv[4].encode("utf-8")
    if not name or len(name) >= NAME_LEN:
        raise SystemExit(f"nickname must be 1..{NAME_LEN - 1} UTF-8 bytes, got {len(name)}")
    for i, base, uuid, old in list(users(data)):
        data[base + NAME_AT:base + NAME_AT + NAME_LEN] = name.ljust(NAME_LEN, b"\0")
        print(f"user {i} uuid {uuid}: {old!r} -> {argv[4]!r}")
    open(argv[3], "wb").write(data)
    print(f"{argv[3]} sha256 {hashlib.sha256(data).hexdigest()}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
