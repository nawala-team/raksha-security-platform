FROM python:3.11-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY src/ ./src/

# Create directories for models and data
RUN mkdir -p /app/models /app/data /app/models/onnx

ENV MODEL_DIR=/app/models
ENV DATA_DIR=/app/data
ENV ML_ENGINE_PORT=8000

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=10s --retries=3 \
    CMD python -c "import httpx; r = httpx.get('http://localhost:8000/health'); r.raise_for_status()"

CMD ["uvicorn", "src.main:app", "--host", "0.0.0.0", "--port", "8000"]
