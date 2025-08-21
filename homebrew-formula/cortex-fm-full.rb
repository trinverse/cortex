class CortexFmFull < Formula
  desc "Modern terminal file manager with dual-pane interface"
  homepage "https://github.com/trinverse/cortex"
  version "0.2.0"
  license "MIT"

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/trinverse/cortex/releases/download/v0.2.0/cortex-0.2.0-x86_64-linux.tar.gz"
      sha256 "PENDING_LINUX_X86_64_SHA256"
    else
      # ARM Linux builds not yet available, build from source
      url "https://github.com/trinverse/cortex/archive/refs/tags/v0.2.0.tar.gz"
      sha256 "PENDING_SOURCE_SHA256"
      depends_on "rust" => :build
    end
  end

  on_macos do
    if Hardware::CPU.arm?
      # macOS ARM binary will be available after GitHub Actions build
      url "https://github.com/trinverse/cortex/releases/download/v0.2.0/cortex-v0.2.0-aarch64-apple-darwin.tar.gz"
      sha256 "399c52146dc2fc94541d5c45369797e9569bfced23bfce1bd4bf009fcc767a96"
    else
      # macOS Intel binary will be available after GitHub Actions build  
      url "https://github.com/trinverse/cortex/releases/download/v0.2.0/cortex-v0.2.0-x86_64-apple-darwin.tar.gz"
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
    assert_match "0.2.0", shell_output("#{bin}/cortex-fm --version")
  end
end