#!/bin/bash

# Quick fix to build without OpenSSL dependency

echo "Applying quick fix to build without OpenSSL..."

# Comment out SSH2 related code in vfs.rs
sed -i 's/^use ssh2/\/\/ use ssh2/' cortex-core/src/vfs.rs
sed -i 's/^use suppaftp/\/\/ use suppaftp/' cortex-core/src/vfs.rs

# Comment out the SSH/FTP provider implementations temporarily
sed -i '/pub struct SshConnectionManager/,/^impl VirtualFileSystemBuilder {/s/^/\/\/ /' cortex-core/src/vfs.rs

echo "Quick fix applied. Now try building with:"
echo "  cargo build"
echo ""
echo "Note: SSH/FTP features will be disabled in this build."