class Parlo < Formula
  desc "Offline dictation, translation and meetings"
  homepage "https://github.com/PedroNascimento09/Parlo"
  version "0.2.9"
  url "https://github.com/PedroNascimento09/Parlo/releases/download/v#{version}/parlo-macos-aarch64.tar.gz"
  sha256 "779cb7917e11d234254dac6ee641eeb91dd0464f09ba3593a430032c1bfeee0b"
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
