"""Pytest fixtures for Raksha ML Engine tests."""

import numpy as np
import pytest


@pytest.fixture
def sample_normal_traffic():
    """Generate synthetic normal network traffic features.
    
    Returns a 2D array of shape (200, 64) representing normal traffic patterns.
    Features are drawn from a multivariate normal distribution to simulate
    baseline network behavior.
    """
    rng = np.random.default_rng(seed=42)
    # Normal traffic: low variance, centered features
    return rng.normal(loc=0.5, scale=0.1, size=(200, 64)).astype(np.float32)


@pytest.fixture
def sample_anomalous_traffic():
    """Generate synthetic anomalous network traffic features.
    
    Returns a 2D array of shape (20, 64) with outlier patterns that
    should be flagged by the anomaly detector.
    """
    rng = np.random.default_rng(seed=99)
    # Anomalous traffic: high variance, shifted distribution
    return rng.normal(loc=2.0, scale=0.8, size=(20, 64)).astype(np.float32)


@pytest.fixture
def sample_feature_names():
    """Return standard feature names for 64-dimensional input."""
    return [f"feature_{i}" for i in range(64)]


@pytest.fixture
def small_training_set():
    """Minimal training set for fast tests (not representative of production)."""
    rng = np.random.default_rng(seed=7)
    return rng.normal(loc=0.5, scale=0.15, size=(50, 16)).astype(np.float32)


@pytest.fixture
def mock_otx_pulse_response():
    """Mock OTX API response for pulse subscription."""
    return {
        "results": [
            {
                "id": "pulse-001",
                "name": "APT28 Campaign Indicators",
                "description": "Indicators associated with recent APT28 activity targeting energy sector",
                "author_name": "AlienVault",
                "created": "2024-03-01T00:00:00Z",
                "modified": "2024-03-15T12:00:00Z",
                "tags": ["apt28", "energy", "spearphishing"],
                "targeted_countries": ["US", "DE", "UA"],
                "attack_ids": [{"id": "T1566"}, {"id": "T1053"}],
                "indicator_count": 47,
            },
            {
                "id": "pulse-002",
                "name": "Ransomware IOCs - March 2024",
                "description": "Collection of ransomware indicators from multiple families",
                "author_name": "Community",
                "created": "2024-03-10T00:00:00Z",
                "modified": "2024-03-14T08:30:00Z",
                "tags": ["ransomware", "lockbit", "blackcat"],
                "targeted_countries": [],
                "attack_ids": [{"id": "T1486"}],
                "indicator_count": 132,
            },
        ],
        "count": 2,
    }


@pytest.fixture
def mock_otx_indicators_response():
    """Mock OTX API response for pulse indicators."""
    return {
        "results": [
            {
                "type": "IPv4",
                "indicator": "198.51.100.23",
                "title": "C2 Server",
                "description": "Known command and control server",
                "created": "2024-03-01T10:00:00Z",
                "is_active": 1,
            },
            {
                "type": "domain",
                "indicator": "malicious-domain.example.com",
                "title": "Phishing domain",
                "description": "Domain used in spearphishing campaign",
                "created": "2024-03-02T14:30:00Z",
                "is_active": 1,
            },
            {
                "type": "FileHash-SHA256",
                "indicator": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                "title": "Malware payload",
                "description": "Ransomware dropper binary",
                "created": "2024-03-03T09:00:00Z",
                "is_active": 0,
            },
        ],
        "count": 3,
    }


@pytest.fixture
def mock_ip_reputation_response():
    """Mock OTX IP reputation lookup response."""
    return {
        "reputation": 42,
        "pulse_info": {"count": 5},
        "country_name": "Russia",
        "asn": "AS12345",
    }


@pytest.fixture
def tmp_model_dir(tmp_path):
    """Provide a temporary directory for model persistence tests."""
    model_dir = tmp_path / "models"
    model_dir.mkdir()
    return str(model_dir)
