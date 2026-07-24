"""Tests for the Anomaly Detector ensemble model."""

import numpy as np
import pytest

from src.models.anomaly_detector import AnomalyDetector, Autoencoder


class TestAutoencoder:
    """Unit tests for the Autoencoder component."""

    def test_initialization(self):
        """Autoencoder initializes with correct dimensions."""
        ae = Autoencoder(input_dim=64, encoding_dim=16)
        assert ae.input_dim == 64
        assert ae.encoding_dim == 16
        assert ae.model is not None
        assert ae.threshold == 0.5

    def test_model_architecture(self):
        """Model has encoder and decoder submodules."""
        ae = Autoencoder(input_dim=32, encoding_dim=8)
        assert hasattr(ae.model, "encoder")
        assert hasattr(ae.model, "decoder")

    def test_forward_pass_shape(self):
        """Forward pass returns tensor with same shape as input."""
        import torch

        ae = Autoencoder(input_dim=64, encoding_dim=16)
        ae.model.eval()
        x = torch.randn(10, 64)
        with torch.no_grad():
            output = ae.model(x)
        assert output.shape == (10, 64)

    def test_fit_updates_threshold(self, small_training_set):
        """Training updates the reconstruction error threshold."""
        ae = Autoencoder(input_dim=16, encoding_dim=4)
        initial_threshold = ae.threshold
        ae.fit(small_training_set, epochs=5, lr=1e-3)
        assert ae.threshold != initial_threshold


class TestAnomalyDetector:
    """Integration tests for the AnomalyDetector ensemble."""

    def test_initialization(self):
        """Detector initializes with default parameters."""
        detector = AnomalyDetector()
        assert detector.is_fitted is False
        assert detector.isolation_forest is not None
        assert detector.autoencoder is not None
        assert detector.scaler is not None

    def test_initialization_custom_params(self):
        """Detector accepts custom autoencoder dimensions."""
        detector = AnomalyDetector(autoencoder_dim=32, encoding_dim=8)
        assert detector.autoencoder.input_dim == 32
        assert detector.autoencoder.encoding_dim == 8

    def test_fit_marks_model_as_fitted(self, sample_normal_traffic, sample_feature_names):
        """fit() sets is_fitted to True."""
        detector = AnomalyDetector(autoencoder_dim=64, encoding_dim=16)
        detector.fit(sample_normal_traffic, feature_names=sample_feature_names, epochs=3)
        assert detector.is_fitted is True

    def test_predict_raises_before_fit(self):
        """predict() raises if model is not fitted."""
        detector = AnomalyDetector()
        features = np.random.randn(1, 64).astype(np.float32)

        with pytest.raises(Exception):
            detector.predict(features)

    def test_predict_normal_traffic(self, sample_normal_traffic, sample_feature_names):
        """Normal traffic should produce valid prediction structure."""
        detector = AnomalyDetector(autoencoder_dim=64, encoding_dim=16)
        detector.fit(sample_normal_traffic, feature_names=sample_feature_names, epochs=10)

        result = detector.predict(sample_normal_traffic[0:1])

        assert "anomaly_score" in result
        assert "is_anomaly" in result
        assert "confidence" in result
        assert "details" in result
        assert 0.0 <= result["anomaly_score"] <= 1.0
        assert isinstance(result["is_anomaly"], bool)

    def test_predict_anomalous_traffic(
        self, sample_normal_traffic, sample_anomalous_traffic, sample_feature_names
    ):
        """Anomalous traffic should have higher anomaly scores than normal."""
        detector = AnomalyDetector(autoencoder_dim=64, encoding_dim=16)
        detector.fit(sample_normal_traffic, feature_names=sample_feature_names, epochs=20)

        normal_result = detector.predict(sample_normal_traffic[0:1])
        anomaly_result = detector.predict(sample_anomalous_traffic[0:1])

        assert anomaly_result["anomaly_score"] > normal_result["anomaly_score"]

    def test_predict_returns_detail_fields(self, sample_normal_traffic, sample_feature_names):
        """Prediction result contains expected detail keys."""
        detector = AnomalyDetector(autoencoder_dim=64, encoding_dim=16)
        detector.fit(sample_normal_traffic, feature_names=sample_feature_names, epochs=3)

        result = detector.predict(sample_normal_traffic[0:1])

        assert "isolation_forest_score" in result["details"]
        assert "autoencoder_error" in result["details"]
        assert "ae_threshold" in result["details"]

    def test_predict_1d_input_handled(self, sample_normal_traffic, sample_feature_names):
        """1D input array is automatically reshaped."""
        detector = AnomalyDetector(autoencoder_dim=64, encoding_dim=16)
        detector.fit(sample_normal_traffic, feature_names=sample_feature_names, epochs=3)

        result = detector.predict(sample_normal_traffic[0])
        assert "anomaly_score" in result

    def test_save_and_load(
        self, sample_normal_traffic, sample_feature_names, tmp_model_dir
    ):
        """Model can be saved and loaded with same predictions."""
        detector = AnomalyDetector(autoencoder_dim=64, encoding_dim=16)
        detector.fit(sample_normal_traffic, feature_names=sample_feature_names, epochs=5)

        test_sample = sample_normal_traffic[0:1]
        result_before = detector.predict(test_sample)

        detector.save(tmp_model_dir)
        loaded = AnomalyDetector.load(tmp_model_dir)
        result_after = loaded.predict(test_sample)

        assert abs(result_before["anomaly_score"] - result_after["anomaly_score"]) < 0.01

    def test_confidence_agreement(self, sample_normal_traffic, sample_feature_names):
        """Confidence should be either 0.6 or 0.9 depending on model agreement."""
        detector = AnomalyDetector(autoencoder_dim=64, encoding_dim=16)
        detector.fit(sample_normal_traffic, feature_names=sample_feature_names, epochs=10)

        result = detector.predict(sample_normal_traffic[0:1])
        assert result["confidence"] in (0.6, 0.9)

