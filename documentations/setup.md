# ball-ball-u Project Setup and Build Workflow

This document provides the complete workflow for setting up, compiling, and running the `ball-ball-u` Rust project on an AlmaLinux (EL8/EL9) CSL machine **without sudo permissions**.

The core problem is that the `libudev` library, a dependency required by `bevy`, must be manually downloaded, extracted, and its path exposed to the Rust compiler (`cargo`).

---

## 1. First-Time Environment Setup (One-Time Only)

The goal of this step is to download the `libudev-devel` package (provided by `systemd-devel` on AlmaLinux 8) and extract it into your `$HOME` directory.

```bash
# 1. Create a directory to hold local dependencies
mkdir -p ~/local_deps
cd ~/local_deps

# 2. Use dnf to download the libudev-devel RPM package
# (This will automatically download systemd-devel-*.rpm, which contains libudev)
dnf download libudev-devel

# 3. Extract the contents of the downloaded .rpm file into the current directory
# This will create a ./usr structure inside ~/local_deps
rpm2cpio *.rpm | cpio -idmv

# 4. (Verify) Check that the key files were extracted to the correct location
# Verify the .pc config file
ls -l ~/local_deps/usr/lib/pkgconfig/libudev.pc
    
# Verify the .so shared library file
ls -l ~/local_deps/usr/lib/libudev.so
```

```bash
# 1. Navigate to your project's workspace root
cd ballballu

# 2. (Critical) Load your environment variables
# The 'source' command executes the export commands from .env in your current shell
source .env

# You should see the "Loading local dependencies..." message

# 3. Now you can build the project normally
cargo build

# 4. (Example) Run your client
# cargo run --package client
```
