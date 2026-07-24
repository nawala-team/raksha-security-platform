"""Tests for Anomaly Detector model."""

import numpy as np
import pandas as pd
import pytest
from ml.src.models.anomaly_detector import AnomalyDetector


@pytest.fixture
def sample_data():
    """Generate sample training data."""
    np.random.seed(42)
    n_samples = 200
    data = pd.DataFrame({
        "cpu_usage": np.random.normal(50, 15, n_samples),
        "memory_usage": np.random.normal(60, 10, n_samples),
        "network_bytes": np.random.normal(1000, 200, n_samples),
        "disk_io": np.random.normal(30, 8, n_samples),
        "login_attempts": np.random.poisson(3, n_samples).astype(float),
    })
    return data


@pytest.fixture
def trained_detector(sample_data, tmp_path):
    """Create and train a detector."""
    detector = AnomalyDetector(model_dir=str(tmp_path), contamination=0.05)
    detector.train(sample_data)
    return detector


def test_training(sample_data, tmp_path):
    detector = AnomalyDetector(model_dir=str(tmp_path))
    result = detector.train(sample_data)
    assert result["training_samples"] == 200
    assert len(result["features"]) == 5
    assert detector.is_trained


def test_training_insufficient_data(tmp_path):
    detector = AnomalyDetector(model_dir=str(tmp_path))
    small_data = pd.DataFrame({"a": [1, 2, 3], "b": [4, 5, 6]})
    with pytest.raises(ValueError, match="at least 50 samples"):
        detector.train(small_data)


def test_predict_normal(trained_detector):
    result = trained_detector.predict("evt-001", {
        "cpu_usage": 50.0,
        "memory_usage": 60.0,
        "network_bytes": 1000.0,
        "disk_io": 30.0,
        "login_attempts": 3.0,
    })
    assert result.event_id == "evt-001"
    assert not result.is_anomaly
    assert 0 <= result.score <= 1


def test_predict_anomaly(trained_detector):
    result = trained_detector.predict("evt-002", {
        "cpu_usage": 200.0,
        "memory_usage": 200.0,
        "network_bytes": 50000.0,
        "disk_io": 200.0,
        "login_attempts": 100.0,
    })
    assert result.event_id == "evt-002"
    assert result.is_anomaly
    assert result.score > 0.3


def test_save_and_load(trained_detector, tmp_path):
    path = trained_detector.save("test_model")
    assert "test_model.joblib" in path

    new_detector = AnomalyDetector(model_dir=str(tmp_path))
    loaded = new_detector.load_model("test_model")
    assert loaded
    assert new_detector.is_trained
    assert new_detector.model_version == trained_detector.model_version


def test_predict_without_training(tmp_path):
    detector = AnomalyDetector(model_dir=str(tmp_path))
    with pytest.raises(RuntimeError, match="not trained"):
        detector.predict("x", {"cpu_usage": 50.0})
