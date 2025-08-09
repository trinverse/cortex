class Cortex < Formula
  desc "Modern terminal file manager with dual-panel interface"
  homepage "https://github.com/trinverse/cortex"
  url "https://github.com/trinverse/cortex/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"
  head "https://github.com/trinverse/cortex.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    # Test that cortex can at least display version
    assert_match "cortex", shell_output("#{bin}/cortex --version")
  end
end