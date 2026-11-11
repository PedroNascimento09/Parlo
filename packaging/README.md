# Package channels

Every channel below repacks the same release downloads: `Parlo-Setup.exe`, `parlo-macos-aarch64.tar.gz` and `parlo-linux-x86_64.tar.gz`, all published with `SHA256SUMS.txt` and a build provenance record. The checksums written into the manifests are the ones published with `v0.2.9`.

Each channel is **Partial**: the manifest is complete and checked against the release workflow, but no install has been run end to end from this machine, so treat it as a recipe that may need a fix before it works.

| Channel | Manifest | Install | What is missing |
| --- | --- | --- | --- |
| winget | `packaging/winget/PedroNascimento09.Parlo.installer.yaml` | `winget install Parlo` | The manifest is in review at [microsoft/winget-pkgs#474924](https://github.com/microsoft/winget-pkgs/pull/474924), so the command resolves when it is accepted |
| Homebrew | `packaging/homebrew/parlo.rb` | `brew install --formula ./packaging/homebrew/parlo.rb` | The formula unpacks the macOS archive instead of building, and `brew audit` and a real install are untried |
| AUR | `packaging/aur/PKGBUILD` | `makepkg -si` inside `packaging/aur` | `parlo-bin` is not uploaded to the AUR, and `makepkg` has not run here |
| Snap | `packaging/snap/snapcraft.yaml` | `snapcraft pack packaging/snap` | Strict confinement is declared but never tried, and no store revision exists |
| DEB | `packaging/deb/debian/rules` | `dpkg-buildpackage -b -uc` inside `packaging/deb` | The rules download and checksum the Linux archive, but no `.deb` is built or installed here |
| AppImage | `packaging/appimage/parlo.appimage.yml` | `appimage-builder --recipe packaging/appimage/parlo.appimage.yml` | The recipe downloads and checksum the Linux archive, but no AppImage is built here |
| Nix | `packaging/nix/default.nix` | `nix build ./packaging/nix` | The derivation unpacks the Linux archive and patches its interpreter, but nothing is built with Nix here |

## Docker

`packaging/docker/Dockerfile` downloads the published Linux archive, checks it against the hash from `SHA256SUMS.txt` before unpacking it, and starts `parlo serve` as the unprivileged user `10001` on port `8787`:

```bash
docker build -t parlo:0.2.9 packaging/docker
docker run --rm -e PARLO_PIN=4821 -p 8787:8787 parlo:0.2.9
curl -H "x-parlo-pin: 4821" http://127.0.0.1:8787/v1/models
```

`parlo serve` binds `127.0.0.1`, so a container could never be reached from outside; `packaging/docker/entrypoint.sh` turns on the same sharing switch the app uses, writing `PARLO_CONFIG` with `share` on, `PARLO_PIN` as the four-to-eight digit PIN and `share_port` equal to `PARLO_PORT`, which is how the binary opens `0.0.0.0` on its own. `PARLO_DATA_DIR` and `PARLO_CONFIG` point at `/data`, kept by the volume.

Docker is **Partial**. This machine has no `docker` and no `podman`, so the image has never been built or run here, the engine inside it is the PyInstaller build without its models, and nothing has been observed about what it answers on `/health`. What is missing: a real `docker build`, a real `docker run`, and the models placed in `/data` before a transcription can be tried.

The Linux archive carries `parlo`, `parlo-app`, `parlo-engine`, the icons and the licence files; the channel manifests keep the two programs together because the app starts the command line binary beside it.

Bumping the version means replacing the version in every manifest and the hashes in the same commit, and `cargo test -p parlo-cli --test packaging` fails until they agree with `Cargo.toml` and with the archives `.github/workflows/release.yml` attaches.
