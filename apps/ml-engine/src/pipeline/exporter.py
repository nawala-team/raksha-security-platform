"""Export trained models to ONNX format for production inference."""

import os
from typing import Any

import numpy as np
import structlog

logger = structlog.get_logger(__name__)


class ONNXExporter:
    """Export sklearn and PyTorch models to ONNX for cross-platform inference."""

    def __init__(self, output_dir: str = "./models/onnx"):
        self.output_dir = output_dir
        os.makedirs(output_dir, exist_ok=True)

    def export_isolation_forest(self, model, scaler, model_name: str) -> str:
        """Export Isolation Forest + scaler pipeline to ONNX."""
        from skl2onnx import to_onnx
        from sklearn.pipeline import Pipeline

        pipeline = Pipeline([("scaler", scaler), ("model", model)])
        sample = np.zeros((1, scaler.n_features_in_), dtype=np.float32)

        onnx_model = to_onnx(pipeline, sample, target_opset=17)
        output_path = os.path.join(self.output_dir, f"{model_name}_if.onnx")

        with open(output_path, "wb") as f:
            f.write(onnx_model.SerializeToString())

        logger.info("onnx_exported", model=model_name, path=output_path)
        return output_path

    def export_autoencoder(self, autoencoder, model_name: str, input_dim: int) -> str:
        """Export PyTorch autoencoder to ONNX."""
        import torch

        autoencoder.model.eval()
        dummy_input = torch.randn(1, input_dim)
        output_path = os.path.join(self.output_dir, f"{model_name}_ae.onnx")

        torch.onnx.export(
            autoencoder.model,
            dummy_input,
            output_path,
            export_params=True,
            opset_version=17,
            do_constant_folding=True,
            input_names=["input"],
            output_names=["output"],
            dynamic_axes={"input": {0: "batch_size"}, "output": {0: "batch_size"}},
        )

        logger.info("onnx_autoencoder_exported", model=model_name, path=output_path)
        return output_path

    def export_anomaly_detector(self, detector) -> dict[str, str]:
        """Export full anomaly detector (both IF and AE) to ONNX."""
        paths = {}
        paths["isolation_forest"] = self.export_isolation_forest(
            detector.isolation_forest, detector.scaler, "anomaly_detector"
        )
        paths["autoencoder"] = self.export_autoencoder(
            detector.autoencoder, "anomaly_detector", detector.autoencoder.input_dim
        )
        return paths

    def export_network_analyzer(self, analyzer) -> str:
        """Export network analyzer to ONNX."""
        return self.export_isolation_forest(
            analyzer.isolation_forest, analyzer.scaler, "network_analyzer"
        )

    def validate_onnx(self, onnx_path: str, sample_input: np.ndarray) -> dict[str, Any]:
        """Validate ONNX model produces correct output."""
        import onnx
        import onnxruntime as ort

        # Validate model structure
        model = onnx.load(onnx_path)
        onnx.checker.check_model(model)

        # Run inference
        session = ort.InferenceSession(onnx_path)
        input_name = session.get_inputs()[0].name
        result = session.run(None, {input_name: sample_input.astype(np.float32)})

        logger.info("onnx_validated", path=onnx_path, output_shape=result[0].shape)
        return {
            "valid": True,
            "output_shape": result[0].shape,
            "path": onnx_path,
        }
