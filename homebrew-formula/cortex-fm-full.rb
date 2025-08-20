class CortexFmFull < Formula
  desc "Modern terminal file manager with dual-pane interface"
  homepage "https://github.com/trinverse/cortex"
  version "0.1.0"
  license "MIT"

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/trinverse/cortex/releases/download/v0.1.0/cortex-0.1.0-x86_64-linux.tar.gz"
      sha256 "4637c013232e2ba5c90fd99e88a569c4b9a2857be764665980e1751533028741"
    else
      # ARM Linux builds not yet available, build from source
      url "https://github.com/trinverse/cortex/archive/refs/tags/v0.1.0.tar.gz"
      sha256 "0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
      depends_on "rust" => :build
    end
  end

  on_macos do
    if Hardware::CPU.arm?
      # macOS ARM binary will be available after GitHub Actions build
      url "https://github.com/trinverse/cortex/releases/download/v0.1.0/cortex-v0.1.0-aarch64-apple-darwin.tar.gz"
      # macOS ARM binary not yet available
      # Remove this section until a valid binary and SHA256 are available
    else
      # macOS Intel binary will be available after GitHub Actions build  
      url "https://github.com/trinverse/cortex/releases/download/v0.1.0/cortex-v0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "PENDING_MACOS_X86_64_SHA256"
    end
  end
  def install
    if (buildpath/"cortex").exist?
      # Pre-built binary
      bin.install "cortex" => "cortex-fm"
    else
      # Build from source
      system "cargo", "install", *std_cargo_args(path: "cortex-cli")
      mv bin/"cortex", bin/"cortex-fm"
    end
  end

  test do
    assert_match "0.1.0", shell_output("#{bin}/cortex-fm --version")
  end
end