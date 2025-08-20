class Cortex < Formula
  desc "Modern terminal file manager with dual-pane interface"
  homepage "https://github.com/trinverse/cortex"
  url "https://github.com/trinverse/cortex/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
  license "MIT"
  
  depends_on "rust" => :build
  
  def install
    system "cargo", "install", *std_cargo_args(path: "cortex-cli")
  end
  
  test do
    assert_match version.to_s, shell_output("#{bin}/cortex --version")
  end
end