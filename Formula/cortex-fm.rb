class CortexFm < Formula
  desc "Modern terminal file manager with dual-pane interface"
  homepage "https://github.com/trinverse/cortex"
  version "0.1.4"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/trinverse/cortex/releases/download/v0.1.4/cortex-v0.1.4-aarch64-apple-darwin.tar.gz"
      sha256 "PENDING_MACOS_ARM64_SHA256"
    else
      # Intel Mac builds from source for now
      url "https://github.com/trinverse/cortex/archive/refs/tags/v0.1.4.tar.gz"
      sha256 "PENDING_MACOS_X86_64_SHA256"
      depends_on "rust" => :build
    end
  end

  on_linux do
    url "https://github.com/trinverse/cortex/archive/refs/tags/v0.1.4.tar.gz"
    sha256 "PENDING_SOURCE_SHA256"  # Will be calculated when tag is pushed
    depends_on "rust" => :build
  end
  
  def install
    if Hardware::CPU.arm? && OS.mac?
      # Pre-built binary for Apple Silicon
      bin.install "cortex" => "cortex-fm"
    else
      # Build from source
      system "cargo", "install", *std_cargo_args(path: "cortex-cli")
      mv bin/"cortex", bin/"cortex-fm"
    end
  end
  
  test do
    assert_match "0.1.4", shell_output("#{bin}/cortex-fm --version")
  end
end