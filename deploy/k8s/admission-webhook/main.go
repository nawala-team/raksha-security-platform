package main

import (
	"crypto/tls"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"strings"

	admissionv1 "k8s.io/api/admission/v1"
	corev1 "k8s.io/api/core/v1"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
)

var (
	tlsCert string
	tlsKey  string
	port    int
)

var allowedRegistries = []string{
	"ghcr.io/raksha-security",
	"docker.io/library",
	"gcr.io/distroless",
	"registry.k8s.io",
}

var sensitiveHostPaths = []string{
	"/etc/shadow",
	"/etc/passwd",
	"/root",
	"/var/run/docker.sock",
	"/var/run/containerd",
	"/proc",
	"/sys",
	"/dev",
}

func init() {
	flag.StringVar(&tlsCert, "tls-cert", "/certs/tls.crt", "Path to TLS certificate")
	flag.StringVar(&tlsKey, "tls-key", "/certs/tls.key", "Path to TLS private key")
	flag.IntVar(&port, "port", 8443, "Webhook server port")
}

func main() {
	flag.Parse()

	if envRegistries := os.Getenv("ALLOWED_REGISTRIES"); envRegistries != "" {
		allowedRegistries = strings.Split(envRegistries, ",")
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/validate", handleValidate)
	mux.HandleFunc("/validate-image", handleValidateImage)
	mux.HandleFunc("/healthz", handleHealth)
	mux.HandleFunc("/readyz", handleHealth)

	cert, err := tls.LoadX509KeyPair(tlsCert, tlsKey)
	if err != nil {
		log.Fatalf("Failed to load TLS keypair: %v", err)
	}

	server := &http.Server{
		Addr:    fmt.Sprintf(":%d", port),
		Handler: mux,
		TLSConfig: &tls.Config{
			Certificates: []tls.Certificate{cert},
			MinVersion:   tls.VersionTLS13,
		},
	}

	log.Printf("Raksha admission webhook starting on port %d", port)
	if err := server.ListenAndServeTLS("", ""); err != nil {
		log.Fatalf("Server failed: %v", err)
	}
}

func handleHealth(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusOK)
	fmt.Fprint(w, "ok")
}

func handleValidate(w http.ResponseWriter, r *http.Request) {
	review, err := parseAdmissionReview(r)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	response := validatePodSecurity(review)
	writeAdmissionResponse(w, review, response)
}

func handleValidateImage(w http.ResponseWriter, r *http.Request) {
	review, err := parseAdmissionReview(r)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	response := validateImagePolicy(review)
	writeAdmissionResponse(w, review, response)
}

func parseAdmissionReview(r *http.Request) (*admissionv1.AdmissionReview, error) {
	if r.Method != http.MethodPost {
		return nil, fmt.Errorf("expected POST, got %s", r.Method)
	}
	body, err := io.ReadAll(r.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read body: %w", err)
	}
	defer r.Body.Close()

	review := &admissionv1.AdmissionReview{}
	if err := json.Unmarshal(body, review); err != nil {
		return nil, fmt.Errorf("failed to unmarshal review: %w", err)
	}
	return review, nil
}

func writeAdmissionResponse(w http.ResponseWriter, review *admissionv1.AdmissionReview, response *admissionv1.AdmissionResponse) {
	review.Response = response
	review.Response.UID = review.Request.UID

	resp, err := json.Marshal(review)
	if err != nil {
		http.Error(w, fmt.Sprintf("failed to marshal response: %v", err), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	w.Write(resp)
}

// validatePodSecurity checks for privileged containers, hostPath mounts,
// and required security context settings.
func validatePodSecurity(review *admissionv1.AdmissionReview) *admissionv1.AdmissionResponse {
	pod, err := extractPodSpec(review)
	if err != nil {
		return allowResponse("unable to parse pod spec, skipping")
	}

	var violations []string

	for _, container := range append(pod.Spec.InitContainers, pod.Spec.Containers...) {
		name := container.Name

		// Check for privileged containers
		if container.SecurityContext != nil {
			if container.SecurityContext.Privileged != nil && *container.SecurityContext.Privileged {
				violations = append(violations,
					fmt.Sprintf("container %q is privileged - privileged containers are not allowed", name))
			}
		}

		// Check runAsNonRoot
		runAsNonRoot := false
		if pod.Spec.SecurityContext != nil && pod.Spec.SecurityContext.RunAsNonRoot != nil {
			runAsNonRoot = *pod.Spec.SecurityContext.RunAsNonRoot
		}
		if container.SecurityContext != nil && container.SecurityContext.RunAsNonRoot != nil {
			runAsNonRoot = *container.SecurityContext.RunAsNonRoot
		}
		if !runAsNonRoot {
			violations = append(violations,
				fmt.Sprintf("container %q must set runAsNonRoot: true", name))
		}

		// Check readOnlyRootFilesystem
		readOnly := false
		if container.SecurityContext != nil && container.SecurityContext.ReadOnlyRootFilesystem != nil {
			readOnly = *container.SecurityContext.ReadOnlyRootFilesystem
		}
		if !readOnly {
			violations = append(violations,
				fmt.Sprintf("container %q must set readOnlyRootFilesystem: true", name))
		}
	}

	// Check for sensitive hostPath mounts
	for _, volume := range pod.Spec.Volumes {
		if volume.HostPath != nil {
			for _, sensitive := range sensitiveHostPaths {
				if strings.HasPrefix(volume.HostPath.Path, sensitive) {
					violations = append(violations,
						fmt.Sprintf("hostPath volume %q mounts sensitive path %q", volume.Name, volume.HostPath.Path))
				}
			}
		}
	}

	if len(violations) > 0 {
		return denyResponse(violations)
	}
	return allowResponse("pod security validation passed")
}

// validateImagePolicy checks image registries and tags.
func validateImagePolicy(review *admissionv1.AdmissionReview) *admissionv1.AdmissionResponse {
	pod, err := extractPodSpec(review)
	if err != nil {
		return allowResponse("unable to parse pod spec, skipping")
	}

	var violations []string

	for _, container := range append(pod.Spec.InitContainers, pod.Spec.Containers...) {
		image := container.Image

		// Check for latest tag
		if strings.HasSuffix(image, ":latest") || !strings.Contains(image, ":") {
			violations = append(violations,
				fmt.Sprintf("container %q uses image %q - latest/untagged images not allowed",
					container.Name, image))
			continue
		}

		// Check allowed registries
		allowed := false
		for _, registry := range allowedRegistries {
			if strings.HasPrefix(image, registry+"/") {
				allowed = true
				break
			}
		}
		if !allowed {
			violations = append(violations,
				fmt.Sprintf("container %q uses image %q from non-approved registry. Allowed: %v",
					container.Name, image, allowedRegistries))
		}
	}

	if len(violations) > 0 {
		return denyResponse(violations)
	}
	return allowResponse("image policy validation passed")
}

func extractPodSpec(review *admissionv1.AdmissionReview) (*corev1.Pod, error) {
	raw := review.Request.Object.Raw
	pod := &corev1.Pod{}
	if err := json.Unmarshal(raw, pod); err != nil {
		return nil, fmt.Errorf("failed to unmarshal pod: %w", err)
	}
	return pod, nil
}

func allowResponse(message string) *admissionv1.AdmissionResponse {
	return &admissionv1.AdmissionResponse{
		Allowed: true,
		Result: &metav1.Status{
			Message: message,
		},
	}
}

func denyResponse(violations []string) *admissionv1.AdmissionResponse {
	message := fmt.Sprintf("Raksha Security Policy Violation(s):\n- %s",
		strings.Join(violations, "\n- "))
	log.Printf("DENIED: %s", message)
	return &admissionv1.AdmissionResponse{
		Allowed: false,
		Result: &metav1.Status{
			Status:  "Failure",
			Message: message,
			Code:    http.StatusForbidden,
		},
	}
}

