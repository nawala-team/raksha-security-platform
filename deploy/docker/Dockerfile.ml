# ─── Stage 1: Builder ────────────────────────────────────────────────
FROM python:3.12-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

COPY ml/pyproject.toml ./
RUN pip install --no-cache-dir --prefix=/install .

# ─── Stage 2: Runtime ────────────────────────────────────────────────
FROM python:3.12-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    tini \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r raksha && useradd -r -g raksha -d /app -s /sbin/nologin raksha

WORKDIR /app

COPY --from=builder /install /usr/local
COPY ml/src/ ./src/

RUN mkdir -p /data/models && chown -R raksha:raksha /app /data

USER raksha

EXPOSE 8000

ENTRYPOINT ["tini", "--"]
CMD ["uvicorn", "src.app:app", "--host", "0.0.0.0", "--port", "8000"]
