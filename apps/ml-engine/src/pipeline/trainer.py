"""Model training pipeline with Optuna hyperparameter optimization."""

import os
import uuid
from typing import Any

import numpy as np
import pandas as pd
import structlog
from datetime import datetime

from src.models.anomaly_detector import AnomalyDetector
from src.models.user_behavior import UserBehaviorAnalyzer
from src.models.network_analyzer import NetworkAnalyzer
from src.pipeline.feature_engineering import FeatureEngineer

logger = structlog.get_logger(__name__)


class TrainingPipeline:
    """Orchestrates model training, evaluation, and persistence."""

    def __init__(self):
        self.jobs: dict[str, dict[str, Any]] = {}
        self.model_dir = os.getenv("MODEL_DIR", "./models")
        self.feature_engineer = FeatureEngineer()

    def create_job(
        self,
        model_type: str,
        data_source: str,
        hyperparameters: dict[str, Any],
        optimize: bool = False,
    ) -> str:
        """Create a training job."""
        job_id = f"train_{model_type}_{uuid.uuid4().hex[:8]}"
        self.jobs[job_id] = {
            "model_type": model_type,
            "data_source": data_source,
            "hyperparameters": hyperparameters,
            "optimize": optimize,
            "status": "created",
            "created_at": datetime.utcnow().isoformat(),
        }
        return job_id

    async def run(self, job_id: str) -> dict[str, Any]:
        """Execute a training job."""
        job = self.jobs.get(job_id)
        if not job:
            return {"success": False, "error": f"Job {job_id} not found"}

        job["status"] = "running"
        logger.info("training_started", job_id=job_id, model_type=job["model_type"])

        try:
            data = await self._load_data(job["data_source"], job["model_type"])
            if data is None:
                return {"success": False, "error": "Failed to load training data"}

            if job["optimize"]:
                best_params = self._optimize_hyperparameters(
                    job["model_type"], data, job["hyperparameters"]
                )
                job["hyperparameters"].update(best_params)

            model = self._train_model(job["model_type"], data, job["hyperparameters"])
            model.save(self.model_dir)

            job["status"] = "completed"
            job["completed_at"] = datetime.utcnow().isoformat()
            return {"success": True, "model": model, "job": job}

        except Exception as e:
            job["status"] = "failed"
            job["error"] = str(e)
            logger.error("training_failed", job_id=job_id, error=str(e))

    async def _load_data(self, data_source: str, model_type: str) -> np.ndarray | None:
        """Load and preprocess training data."""
        try:
            if data_source == "latest":
                # Load from default data directory
                data_dir = os.getenv("DATA_DIR", "./data")
                path = os.path.join(data_dir, f"{model_type}_training.parquet")
                if os.path.exists(path):
                    df = pd.read_parquet(path)
                    return self.feature_engineer.extract_from_dataframe(df, model_type)
                # Generate synthetic data for initial training
                return self._generate_synthetic_data(model_type)
            elif os.path.exists(data_source):
                df = pd.read_parquet(data_source)
                return self.feature_engineer.extract_from_dataframe(df, model_type)
            else:
                logger.error("data_source_not_found", source=data_source)
                return None
        except Exception as e:
            logger.error("data_load_failed", error=str(e))
            return None

    def _generate_synthetic_data(self, model_type: str) -> np.ndarray:
        """Generate synthetic training data for bootstrapping."""
        rng = np.random.default_rng(42)
        n_samples = 10000
        if model_type == "anomaly_detector":
            return rng.normal(0, 1, (n_samples, 64))
        elif model_type == "user_behavior":
            return rng.normal(0, 1, (n_samples, 16))
        else:
            return rng.normal(0, 1, (n_samples, 16))

    def _train_model(self, model_type: str, data: np.ndarray, params: dict[str, Any]):
        """Instantiate and train the appropriate model."""
        if model_type == "anomaly_detector":
            model = AnomalyDetector(
                contamination=params.get("contamination", 0.05),
                n_estimators=params.get("n_estimators", 200),
                autoencoder_dim=data.shape[1],
            )
            model.fit(data)
        elif model_type == "user_behavior":
            model = UserBehaviorAnalyzer(
                contamination=params.get("contamination", 0.03),
            )
            model.fit(data)
        elif model_type == "network_analyzer":
            model = NetworkAnalyzer(
                contamination=params.get("contamination", 0.05),
            )
            model.fit(data)
        else:
            raise ValueError(f"Unknown model type: {model_type}")
        return model

    def _optimize_hyperparameters(
        self, model_type: str, data: np.ndarray, base_params: dict[str, Any]
    ) -> dict[str, Any]:
        """Run Optuna hyperparameter optimization."""
        import optuna

        optuna.logging.set_verbosity(optuna.logging.WARNING)

        def objective(trial):
            contamination = trial.suggest_float("contamination", 0.01, 0.1)
            n_estimators = trial.suggest_int("n_estimators", 100, 500, step=50)

            # Use cross-validation score as objective
            from sklearn.model_selection import cross_val_score
            from sklearn.ensemble import IsolationForest

            model = IsolationForest(
                n_estimators=n_estimators,
                contamination=contamination,
                random_state=42,
            )
            scores = cross_val_score(model, data, cv=3, scoring="neg_mean_squared_error")
            return scores.mean()

        study = optuna.create_study(direction="maximize")
        study.optimize(objective, n_trials=20, timeout=300)

        logger.info("optimization_complete", best_params=study.best_params)
        return study.best_params

            return {"success": False, "error": str(e)}
