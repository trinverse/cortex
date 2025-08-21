class CortexFm < Formula
  desc "Modern terminal file manager with dual-pane interface"
  homepage "https://github.com/trinverse/cortex"
  version "0.2.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/trinverse/cortex/releases/download/v0.2.0/cortex-v0.2.0-aarch64-apple-darwin.tar.gz"
      sha256 "399c52146dc2fc94541d5c45369797e9569bfced23bfce1bd4bf009fcc767a96"
    else
      # Intel Mac builds from source for now
      url "https://github.com/trinverse/cortex/archive/refs/tags/v0.2.0.tar.gz"
      sha256 "f52e7fd8f76dc74a2c43dd73a5b4903e339c75b002e94a172902f63199450f9f"
      depends_on "rust" => :build
    end
  end

  on_linux do
    url "https://github.com/trinverse/cortex/archive/refs/tags/v0.2.0.tar.gz"
    sha256 "f52e7fd8f76dc74a2c43dd73a5b4903e339c75b002e94a172902f63199450f9f"
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
    assert_match "0.2.0", shell_output("#{bin}/cortex-fm --version")
  end
end