#!/usr/bin/env bash
# Builds the real castle:local image (multi-stage Dockerfile in this
# directory, matching this host's installed rustc, see Dockerfile's own
# comment) and loads it into the platform-eng-colima kind cluster's node
# containerd -- `kind load docker-image` tried first (works here: this is
# a single-platform local build, unlike services/vuln-scanner's
# multi-platform `aquasec/trivy` case that needed the docker save/cp/ctr
# import fallback), falling back to that same fallback path if it ever
# doesn't.
set -euo pipefail

IMAGE="${CASTLE_IMAGE:-castle:local}"
KIND_CLUSTER="${KIND_CLUSTER:-platform-eng-colima}"
KIND_NODE="${KIND_NODE:-platform-eng-colima-control-plane}"

cd "$(dirname "${BASH_SOURCE[0]}")"

echo "==> docker build -t ${IMAGE} ."
docker build -t "${IMAGE}" .

echo "==> kind load docker-image ${IMAGE} --name ${KIND_CLUSTER}"
if kind load docker-image "${IMAGE}" --name "${KIND_CLUSTER}"; then
  echo "==> loaded via kind load docker-image"
else
  echo "==> kind load docker-image failed, falling back to save/cp/ctr-import"
  TMP_TAR="$(mktemp -t castle-image-XXXXXX.tar)"
  trap 'rm -f "${TMP_TAR}"' EXIT
  docker save "${IMAGE}" -o "${TMP_TAR}"
  docker cp "${TMP_TAR}" "${KIND_NODE}:/root/castle-image.tar"
  docker exec "${KIND_NODE}" ctr --namespace=k8s.io images import /root/castle-image.tar
  docker exec "${KIND_NODE}" rm -f /root/castle-image.tar
fi

echo "==> verifying image is present in the node's containerd"
docker exec "${KIND_NODE}" crictl images | grep -i castle
