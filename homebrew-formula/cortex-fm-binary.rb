class CortexFmBinary < Formula
  desc "Modern terminal file manager with dual-pane interface"
  homepage "https://github.com/trinverse/cortex"
  version "0.1.0"
  license "MIT"

  on_linux do
    url "https://github.com/trinverse/cortex/releases/download/v0.1.0/cortex-0.1.0-x86_64-linux.tar.gz"
    sha256 "4637c013232e2ba5c90fd99e88a569c4b9a2857be764665980e1751533028741"
  end

  on_macos do
    # For now, build from source on macOS
    url "https://github.com/trinverse/cortex/archive/refs/tags/v0.1.0.tar.gz"
    sha256 "0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
    
    depends_on "rust" => :build
  end

  def install
    if OS.mac?
      system "cargo", "install", *std_cargo_args(path: "cortex-cli")
      mv bin/"cortex", bin/"cortex-fm"
    else
      bin.install "cortex" => "cortex-fm"
    end
  end

  test do
    assert_match "0.1.0", shell_output("#{bin}/cortex-fm --version")
  end
end