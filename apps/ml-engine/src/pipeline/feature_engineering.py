"""Feature engineering for security event data."""

from typing import Any

import numpy as np
import pandas as pd
import structlog

logger = structlog.get_logger(__name__)


class FeatureEngineer:
    """Extract and transform features from raw security metrics."""

    def transform(self, raw_features: dict[str, Any], model_type: str) -> np.ndarray:
        """Transform raw feature dict into model-ready numpy array."""
        if model_type == "anomaly_detector":
            return self._transform_anomaly_features(raw_features)
        elif model_type == "user_behavior":
            return self._transform_user_features(raw_features)
        elif model_type == "network_analyzer":
            return self._transform_network_features(raw_features)
        else:
            raise ValueError(f"Unknown model type: {model_type}")

    def extract_from_dataframe(self, df: pd.DataFrame, model_type: str) -> np.ndarray:
        """Extract features from a pandas DataFrame."""
        if model_type == "anomaly_detector":
            return self._extract_anomaly_features(df)
        elif model_type == "user_behavior":
            return self._extract_user_features(df)
        elif model_type == "network_analyzer":
            return self._extract_network_features(df)
        else:
            raise ValueError(f"Unknown model type: {model_type}")

    def _transform_anomaly_features(self, raw: dict[str, Any]) -> np.ndarray:
        """Generic anomaly detection features."""
        feature_keys = [
            "cpu_usage", "memory_usage", "disk_io_read", "disk_io_write",
            "network_in", "network_out", "process_count", "open_connections",
            "error_rate", "request_latency_p50", "request_latency_p95",
            "request_latency_p99", "active_sessions", "queue_depth",
            "cache_hit_ratio", "thread_count",
        ]
        features = [float(raw.get(k, 0.0)) for k in feature_keys]
        # Pad to expected dimension if needed
        while len(features) < 64:
            features.append(0.0)
        return np.array(features[:64], dtype=np.float32)

    def _transform_user_features(self, raw: dict[str, Any]) -> np.ndarray:
        """User behavior analytics features."""
        from src.models.user_behavior import UserBehaviorAnalyzer
        features = [float(raw.get(k, 0.0)) for k in UserBehaviorAnalyzer.FEATURE_NAMES]
        return np.array(features, dtype=np.float32)

    def _transform_network_features(self, raw: dict[str, Any]) -> np.ndarray:
        """Network traffic features."""
        from src.models.network_analyzer import NetworkAnalyzer
        features = [float(raw.get(k, 0.0)) for k in NetworkAnalyzer.FEATURE_NAMES]
        return np.array(features, dtype=np.float32)

    def _extract_anomaly_features(self, df: pd.DataFrame) -> np.ndarray:
        """Extract anomaly features from DataFrame."""
        numeric_cols = df.select_dtypes(include=[np.number]).columns.tolist()
        if len(numeric_cols) >= 64:
            return df[numeric_cols[:64]].values.astype(np.float32)
        # Pad with zeros
        data = df[numeric_cols].values.astype(np.float32)
        padding = np.zeros((data.shape[0], 64 - data.shape[1]), dtype=np.float32)
        return np.hstack([data, padding])

    def _extract_user_features(self, df: pd.DataFrame) -> np.ndarray:
        """Extract UBA features from DataFrame."""
        from src.models.user_behavior import UserBehaviorAnalyzer
        cols = [c for c in UserBehaviorAnalyzer.FEATURE_NAMES if c in df.columns]
        if cols:
            data = df[cols].values.astype(np.float32)
            if data.shape[1] < 16:
                padding = np.zeros((data.shape[0], 16 - data.shape[1]), dtype=np.float32)
                data = np.hstack([data, padding])
            return data
        return df.select_dtypes(include=[np.number]).values[:, :16].astype(np.float32)

    def _extract_network_features(self, df: pd.DataFrame) -> np.ndarray:
        """Extract network features from DataFrame."""
        from src.models.network_analyzer import NetworkAnalyzer
        cols = [c for c in NetworkAnalyzer.FEATURE_NAMES if c in df.columns]
        if cols:
            data = df[cols].values.astype(np.float32)
            if data.shape[1] < 16:
                padding = np.zeros((data.shape[0], 16 - data.shape[1]), dtype=np.float32)
                data = np.hstack([data, padding])
            return data
        return df.select_dtypes(include=[np.number]).values[:, :16].astype(np.float32)
