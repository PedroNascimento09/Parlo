class Parlo < Formula
  desc "Offline dictation, translation and meetings"
  homepage "https://github.com/PedroNascimento09/Parlo"
  version "0.2.5"
  url "https://github.com/PedroNascimento09/Parlo/releases/download/v#{version}/parlo-macos-aarch64.tar.gz"
  sha256 "8018d7b396619f9e217d0b2743c6b01eeaeaea798bbb61761e8093ca95e7be10"
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
