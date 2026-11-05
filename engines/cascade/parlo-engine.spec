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
    "yt_dlp",
]

conversion_only = ("ctranslate2.converters", "ctranslate2.specs")

for package in (
    "sherpa_onnx",
    "faster_whisper",
    "ctranslate2",
    "tokenizers",
    "huggingface_hub",
    "yt_dlp",
):
    try:
        package_datas, package_binaries, package_hidden = collect_all(package)
    except Exception:
        continue
    datas += package_datas
    binaries += package_binaries
    hiddenimports += [
        name for name in package_hidden if not name.startswith(conversion_only)
    ]

dropped = (
    "torch",
    "torchaudio",
    "torchvision",
    "onnxruntime",
    "tensorflow",
    "jax",
    "transformers",
    "PIL",
    "hf_xet",
)


unused = ("silero_vad_v6.onnx", "sherpa-onnx-cxx-api.dll")


def kept(entry: str) -> bool:
    path = Path(entry)
    if path.name.startswith("cudnn") or path.name in unused:
        return False
    if path.suffix in (".py", ".pyi", ".lib"):
        return False
    if path.parent.name in ("converters", "specs", "models"):
        return False
    return not any(part in dropped for part in path.parts)


binaries = [entry for entry in binaries if kept(entry[0])]
datas = [entry for entry in datas if kept(entry[0])]

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
        "torch",
        "torchaudio",
        "torchvision",
        "onnxruntime",
        "tensorflow",
        "jax",
        "transformers",
        "PIL",
        "hf_xet",
        "av",
        "pandas",
        "spacy",
        "thinc",
        "weasel",
        "pydantic",
        "pydantic_core",
        "cryptography",
        "rich",
        "pygments",
        "click",
        "setuptools",
        "jinja2",
        "safetensors",
        "google",
        "sqlite3",
    ],
    noarchive=False,
    optimize=2,
)

for collected in (analysis.binaries, analysis.datas):
    collected[:] = [entry for entry in collected if kept(entry[0])]

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
