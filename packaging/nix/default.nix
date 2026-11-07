{ lib, stdenv, fetchurl, autoPatchelfHook, alsa-lib, gtk3, webkitgtk_4_1 }:

stdenv.mkDerivation rec {
  pname = "parlo";
  version = "0.2.6";

  src = fetchurl {
    url = "https://github.com/PedroNascimento09/Parlo/releases/download/v${version}/parlo-linux-x86_64.tar.gz";
    hash = "sha256-VgDThwAdIsJkcsLuxva8PQfvkG8GpCy343MrJ5hxV/4=";
  };

  nativeBuildInputs = [ autoPatchelfHook ];
  buildInputs = [ alsa-lib gtk3 webkitgtk_4_1 stdenv.cc.cc.lib ];

  dontBuild = true;
  dontConfigure = true;

  installPhase = ''
    runHook preInstall
    install -Dm755 parlo $out/bin/parlo
    install -d $out/lib/parlo
    cp -r parlo-app parlo-engine parlo.png parlo.ico $out/lib/parlo/
    ln -s $out/lib/parlo/parlo-app $out/bin/parlo-app
    install -Dm644 parlo.png $out/share/icons/hicolor/512x512/apps/parlo.png
    install -Dm644 LICENSE $out/share/licenses/parlo/LICENSE
    runHook postInstall
  '';

  meta = with lib; {
    description = "Offline dictation, translation and meetings";
    homepage = "https://github.com/PedroNascimento09/Parlo";
    license = licenses.asl20;
    platforms = [ "x86_64-linux" ];
    mainProgram = "parlo";
    sourceProvenance = [ sourceTypes.binaryNativeCode ];
  };
}
