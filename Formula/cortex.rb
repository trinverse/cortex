class Cortex < Formula
  desc "Modern terminal file manager with dual-pane interface"
  homepage "https://github.com/trinverse/cortex"
  version "0.1.4"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/trinverse/cortex/releases/download/v0.1.4/cortex-0.1.4-macos-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_AARCH64"
    else
      url "https://github.com/trinverse/cortex/releases/download/v0.1.4/cortex-0.1.4-macos-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64"
    end
  end

  on_linux do
    if Hardware::CPU.arm? && Hardware::CPU.is_64_bit?
      url "https://github.com/trinverse/cortex/releases/download/v0.1.4/cortex-0.1.4-linux-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_AARCH64"
    else
      url "https://github.com/trinverse/cortex/releases/download/v0.1.4/cortex-0.1.4-linux-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X86_64"
    end
  end

  def install
    bin.install "cortex"
  end

  test do
    assert_match "Cortex v#{version}", shell_output("#{bin}/cortex --version")
  end
end