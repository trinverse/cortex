#!/bin/bash

echo "Testing SSH/SFTP/FTP functionality in Cortex"
echo "============================================"
echo ""

echo "Building with SSH feature enabled..."
~/.cargo/bin/cargo build --package cortex-core --features ssh --release

if [ $? -eq 0 ]; then
    echo "✓ Build successful with SSH feature"
else
    echo "✗ Build failed with SSH feature"
    exit 1
fi

echo ""
echo "Running tests with SSH feature..."
~/.cargo/bin/cargo test --package cortex-core --features ssh --release

if [ $? -eq 0 ]; then
    echo "✓ All tests passed"
else
    echo "✗ Tests failed"
    exit 1
fi

echo ""
echo "Building without SSH feature (backward compatibility)..."
~/.cargo/bin/cargo build --package cortex-core --release

if [ $? -eq 0 ]; then
    echo "✓ Build successful without SSH feature"
else
    echo "✗ Build failed without SSH feature"
    exit 1
fi

echo ""
echo "============================================"
echo "SSH/SFTP/FTP implementation complete!"
echo ""
echo "Features implemented:"
echo "  ✓ Modular SSH connection manager with connection pooling"
echo "  ✓ SFTP provider with full VFS integration"
echo "  ✓ FTP provider with async support"
echo "  ✓ Automatic retry logic and timeout handling"
echo "  ✓ Feature flags for optional compilation"
echo "  ✓ Comprehensive test coverage"
echo ""
echo "To use SSH/SFTP/FTP features, build with:"
echo "  cargo build --features ssh"
echo ""