# Package channels

Every channel below repacks the same release downloads: `Parlo-Setup.exe`, `parlo-macos-aarch64.tar.gz` and `parlo-linux-x86_64.tar.gz`, all published with `SHA256SUMS.txt` and a build provenance record. The checksums written into the manifests are the ones published with `v0.2.4`.

Each channel is **Partial**: the manifest is complete and checked against the release workflow, but no install has been run end to end from this machine, so treat it as a recipe that may need a fix before it works.

| Channel | Manifest | Install | What is missing |
| --- | --- | --- | --- |
| winget | `packaging/winget/PedroNascimento09.Parlo.installer.yaml` | `winget install PedroNascimento09.Parlo` | Nothing is submitted to the winget repository, so the command fails until it is accepted |
| Homebrew | `packaging/homebrew/parlo.rb` | `brew install --formula ./packaging/homebrew/parlo.rb` | The formula unpacks the macOS archive instead of building, and `brew audit` and a real install are untried |
| AUR | `packaging/aur/PKGBUILD` | `makepkg -si` inside `packaging/aur` | `parlo-bin` is not uploaded to the AUR, and `makepkg` has not run here |
| Snap | `packaging/snap/snapcraft.yaml` | `snapcraft pack packaging/snap` | Strict confinement is declared but never tried, and no store revision exists |
| DEB | `packaging/deb/debian/rules` | `dpkg-buildpackage -b -uc` inside `packaging/deb` | The rules download and checksum the Linux archive, but no `.deb` is built or installed here |
| AppImage | `packaging/appimage/parlo.appimage.yml` | `appimage-builder --recipe packaging/appimage/parlo.appimage.yml` | The recipe downloads and checksum the Linux archive, but no AppImage is built here |
| Nix | `packaging/nix/default.nix` | `nix build ./packaging/nix` | The derivation unpacks the Linux archive and patches its interpreter, but nothing is built with Nix here |

The Linux archive carries `parlo`, `parlo-app`, `parlo-engine`, the icons and the licence files; the channel manifests keep the two programs together because the app starts the command line binary beside it.

Bumping the version means replacing the version in every manifest and the hashes in the same commit, and `cargo test -p parlo-cli --test packaging` fails until they agree with `Cargo.toml` and with the archives `.github/workflows/release.yml` attaches.
