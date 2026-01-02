class Rift < Formula
  desc "P2P localhost tunneling — your teammate's ports on your machine"
  homepage "https://github.com/n33levo/rift"
  url "https://github.com/n33levo/rift/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "2b4375a0e8318cf2905911cbd0dac1ac101d75cad2e8e33159fdd2ffb907241c"
  license "MIT"
  head "https://github.com/n33levo/rift.git", branch: "master"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--root", prefix, "--path", "crates/wh-cli"
  end

  test do
    assert_match "rift", shell_output("#{bin}/rift --version")
  end
end
