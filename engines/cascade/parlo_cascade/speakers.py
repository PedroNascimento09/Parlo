from pathlib import Path

from .models import models_dir

MODEL = "3dspeaker_speech_campplus_sv_en_voxceleb_16k.onnx"
SAMPLE_RATE = 16_000
MIN_SECONDS = 1.0


class MissingSpeakerModel(RuntimeError):
    pass


def model_path(root=None) -> Path:
    return (Path(root) if root else models_dir()) / MODEL


def installed(root=None) -> bool:
    return model_path(root).is_file()


class Embedder:
    def __init__(self, root=None):
        path = model_path(root)
        if not path.is_file():
            raise MissingSpeakerModel(
                "Speaker separation is not installed; add it in Model library"
            )
        import sherpa_onnx

        self.path = path
        self.extractor = sherpa_onnx.SpeakerEmbeddingExtractor(
            sherpa_onnx.SpeakerEmbeddingExtractorConfig(
                model=str(path), num_threads=1, debug=False
            )
        )

    def __call__(self, samples) -> list[float]:
        import numpy as np

        audio = np.asarray(samples, dtype=np.float32)
        if len(audio) < SAMPLE_RATE * MIN_SECONDS:
            raise ValueError("a voice is measured from at least a second of speech")
        stream = self.extractor.create_stream()
        stream.accept_waveform(SAMPLE_RATE, audio)
        stream.input_finished()
        if not self.extractor.is_ready(stream):
            raise ValueError("that window carries too little speech to tell who spoke")
        return [float(value) for value in self.extractor.compute_embedding(stream)]
