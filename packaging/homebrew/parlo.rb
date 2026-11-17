class Parlo < Formula
  desc "Offline dictation, translation and meetings"
  homepage "https://github.com/PedroNascimento09/Parlo"
  version "0.2.12"
  url "https://github.com/PedroNascimento09/Parlo/releases/download/v#{version}/parlo-macos-aarch64.tar.gz"
  sha256 "3b7085e3b160833664b3477831b5b30102602fe2f5d9889aceaa11471094634d"
  license "Apache-2.0"

  depends_on arch: :arm64

  def install
    bin.install "parlo"
    libexec.install "parlo-app", "parlo-engine", "parlo.png", "parlo.ico"
  end

  test do
    system "#{bin}/parlo", "--version"
  end
end
