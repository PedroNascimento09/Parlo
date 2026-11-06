class Parlo < Formula
  desc "Offline dictation, translation and meetings"
  homepage "https://github.com/PedroNascimento09/Parlo"
  version "0.2.5"
  url "https://github.com/PedroNascimento09/Parlo/releases/download/v#{version}/parlo-macos-aarch64.tar.gz"
  sha256 "a5664c446a558e03d151a4ba36aea46153965453377fe6025457bfe0b6e30da4"
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
