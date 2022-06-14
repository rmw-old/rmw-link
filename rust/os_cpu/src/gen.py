#!/usr/bin/env python3

OS = "windows macos ios linux android freebsd dragonfly openbsd netbsd".split()
CPU = "x86 x86_64 mips powerpc powerpc64 arm aarch64".split()

OS_CPU = dict(
    windows="x86 x86_64 aarch64",
    macos="x86_64 aarch64",
    ios="aarch64",
    android="aarch64",
    linux="mips x86 x86_64 aarch64"
    )
OS_CPU['freebsd'] = OS_CPU['linux']

n = 0
print("pub const ID:[u8;2] = {")
for os,v in OS_CPU.items():
  assert(os in OS)
  for cpu in v.split():
    assert(cpu in CPU)
    n+=1
    print(f"""#[cfg(target_os = "{os}")]
#[cfg(target_arch = "{cpu}")]""")
    print("{%du16}"%n)

print("}.to_le_bytes();")
