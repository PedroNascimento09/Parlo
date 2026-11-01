from pathlib import Path

from PyInstaller.utils.hooks import collect_all

root = Path(SPECPATH)

datas = [
    (str(root / "glossary.txt"), "."),
    (str(root / "scripts" / "setup_models.py"), "scripts"),
]
binaries = []
hiddenimports = [
    "serve",
    "parlo_cascade",
    "parlo_cascade.engine",
    "parlo_hibiki",
    "parlo_hibiki.protocol",
    "truststore",
    "numpy",
    "wave",
    "faster_whisper",
]

for package in (
    "sherpa_onnx",
    "onnxruntime",
    "faster_whisper",
    "ctranslate2",
    "tokenizers",
    "huggingface_hub",
):
    try:
        package_datas, package_binaries, package_hidden = collect_all(package)
    except Exception:
        continue
    datas += package_datas
    binaries += package_binaries
    hiddenimports += package_hidden

analysis = Analysis(
    ["parlo_engine.py"],
    pathex=[str(root), str(root.parent / "hibiki")],
    binaries=binaries,
    datas=datas,
    hiddenimports=hiddenimports,
    excludes=[
        "tkinter",
        "matplotlib",
        "pytest",
        "IPython",
        "torchaudio",
        "torchvision",
        "av",
        "pandas",
        "matplotlib",
    ],
    noarchive=False,
)

pyz = PYZ(analysis.pure)

exe = EXE(
    pyz,
    analysis.scripts,
    [],
    exclude_binaries=True,
    name="parlo-engine",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=False,
    noupx=True,
    console=True,
)

COLLECT(
    exe,
    analysis.binaries,
    analysis.datas,
    strip=False,
    upx=False,
    name="parlo-engine",
)
