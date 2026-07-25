# Container Security

> Kubernetes admission control, image scanning, and runtime monitoring

## Overview

Raksha provides container security through three layers: admission control (prevent), image scanning (detect), and runtime monitoring (respond).

## Features

### 1. Kubernetes Admission Webhook

Prevents insecure workloads from being deployed:

- Block privileged containers
- Enforce resource limits
- Require security contexts
- Validate image registries (allowlist)
- Reject containers running as root
- Enforce read-only root filesystem

### 2. Image Scanning

- Scan container images for known CVEs
- Check base image freshness
- Detect embedded secrets and credentials
- Verify image signatures (cosign/notation)
- Integration with registry webhooks

### 3. Runtime Monitoring

- Detect container escape attempts
- Monitor network connections from pods
- Alert on unexpected process execution
- File integrity monitoring inside containers
- Resource abuse detection (cryptomining)

## Admission Webhook Setup

### Deploy via Helm

```bash
helm install raksha-admission deploy/k8s/helm/admission-webhook \
  --set portalUrl=https://your-portal \
  --set token=rkat_xxx \
  --namespace raksha-system \
  --create-namespace
```

### Policy Configuration

```yaml
# configs/container-security/admission-policy.yml
policies:
  - name: no-privileged
    action: deny
    match:
      securityContext:
        privileged: true
    message: "Privileged containers are not allowed"

  - name: require-limits
    action: deny
    match:
      resources:
        limits: null
    message: "Resource limits are required"

  - name: allowed-registries
    action: deny
    match:
      image:
        not_in:
          - ghcr.io/yourorg/*
          - docker.io/library/*
    message: "Image from unauthorized registry"

  - name: no-root
    action: deny
    match:
      securityContext:
        runAsNonRoot: false
    message: "Containers must not run as root"
```

## Image Scanning

### On-Demand Scan

```bash
curl -X POST http://localhost:8080/api/v1/container/scan \
  -H "Authorization: Bearer <token>" \
  -d '''{"image": "nginx:latest", "registry": "docker.io"}'''
```

### Registry Webhook

Automatically scan new images pushed to your registry:

```yaml
# configs/container-security/registry-webhook.yml
registries:
  - url: ghcr.io/yourorg
    auth: env:GHCR_TOKEN
    scan_on_push: true
    block_critical: true
    max_cvss_score: 7.0
```

## Supported Platforms

| Platform | Admission | Scanning | Runtime |
|----------|:---------:|:--------:|:-------:|
| Kubernetes | Yes | Yes | Yes |
| Docker Standalone | - | Yes | Yes |
| Amazon ECS | - | Yes | Partial |
| Google GKE | Yes | Yes | Yes |
| Azure AKS | Yes | Yes | Yes |

---

*Part of the Nawala Ecosystem*
