# Build stage
FROM golang:1.22-alpine AS builder

RUN apk add --no-cache git ca-certificates

WORKDIR /build

COPY go.mod go.sum ./
RUN go mod download

COPY . .

RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build \
    -ldflags="-w -s -X main.version=$(git describe --tags --always 2>/dev/null || echo dev)" \
    -trimpath \
    -o /build/webhook \
    .

# Runtime stage
FROM gcr.io/distroless/static:nonroot

LABEL org.opencontainers.image.title="Raksha Admission Webhook"
LABEL org.opencontainers.image.description="Kubernetes admission webhook for container security policy enforcement"
LABEL org.opencontainers.image.vendor="Raksha Security"

COPY --from=builder /build/webhook /webhook

USER 65534:65534

EXPOSE 8443

ENTRYPOINT ["/webhook"]
