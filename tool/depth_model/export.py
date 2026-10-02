# /// script
# requires-python = ">=3.12,<3.13"
# dependencies = [
#   "onnx==1.23.1",
#   "onnxruntime==1.30.0",
#   "onnxscript==0.7.2",
#   "torch==2.14.1",
#   "transformers==5.18.0",
# ]
# ///
"""Converts the depth model the game reads rooms with into the file it ships.

Run through the Dart tool, which checks the result:

    dart tool/build_depth_model.dart

The model is Depth Anything V2 (small), fine-tuned by its authors on indoor
scenes to answer in metres, taken from their own repository at a pinned
revision. It is released under Apache-2.0.

Two things are done to it:

- It is traced for one picture size, the one the game cuts camera frames
  down to, and saved as ONNX, which the phone can run.
- Its weights are stored as 8-bit numbers instead of 32-bit ones. That
  makes the file 27 MB instead of 99 MB and nearly doubles its speed on a
  phone, for no loss of accuracy that showed up against rooms of known
  depth (see docs/environment.md).

This is Python because the model's own tooling is. It is not part of the
game or its tools, and nothing imports it.
"""

import argparse
import logging
import pathlib
import tempfile
import warnings

import torch
from onnxruntime.quantization import QuantType, quantize_dynamic
from transformers import DepthAnythingForDepthEstimation

REPOSITORY = "depth-anything/Depth-Anything-V2-Metric-Indoor-Small-hf"
REVISION = "8078d68a9c75a972131914f6afd0c1723be0da7f"


class DepthOnly(torch.nn.Module):
    """The model with one plain input and one plain output."""

    def __init__(self, model: torch.nn.Module) -> None:
        super().__init__()
        self.model = model

    def forward(self, pixel_values: torch.Tensor) -> torch.Tensor:
        return self.model(pixel_values=pixel_values).predicted_depth


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("output", type=pathlib.Path)
    parser.add_argument("--width", type=int, required=True)
    parser.add_argument("--height", type=int, required=True)
    arguments = parser.parse_args()

    # The tracer and the quantizer both warn at length about choices that
    # are deliberate here: a fixed picture size, and no calibration data.
    warnings.filterwarnings("ignore")
    logging.disable(logging.WARNING)

    model = DepthAnythingForDepthEstimation.from_pretrained(
        REPOSITORY,
        revision=REVISION,
    ).eval()
    picture = torch.zeros(1, 3, arguments.height, arguments.width)

    with tempfile.TemporaryDirectory() as scratch:
        traced = pathlib.Path(scratch) / "traced.onnx"

        torch.onnx.export(
            DepthOnly(model),
            (picture,),
            str(traced),
            input_names=["pixel_values"],
            output_names=["predicted_depth"],
            opset_version=17,
            dynamo=False,
        )

        arguments.output.parent.mkdir(parents=True, exist_ok=True)
        quantize_dynamic(
            str(traced),
            str(arguments.output),
            weight_type=QuantType.QUInt8,
        )


if __name__ == "__main__":
    main()
