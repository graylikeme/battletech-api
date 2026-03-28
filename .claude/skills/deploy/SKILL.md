---
name: deploy
description: "Build, push, and deploy to production. Usage: /deploy api | /deploy roster | /deploy (defaults to api)"
disable-model-invocation: true
allowed-tools: Bash, Read
---

Deploy a service to production. Accepts one argument: `api` (default) or `roster`.

## Service: api

Run from the battletech repo root. Stop on any failure:

1. Push commits to origin:
   ```
   git push origin main
   ```

2. Build the amd64 Docker image and push to GHCR:
   ```
   docker buildx build --platform linux/amd64 \
     -t ghcr.io/graylikeme/battletech-api:sha-$(git rev-parse --short HEAD) \
     -t ghcr.io/graylikeme/battletech-api:latest \
     --push .
   ```

3. Restart the K8s deployment to pull the new image:
   ```
   KUBECONFIG=~/.kube/battletech.yaml kubectl rollout restart deployment/battletech-api -n battletech
   ```

4. Wait for rollout to complete:
   ```
   KUBECONFIG=~/.kube/battletech.yaml kubectl rollout status deployment/battletech-api -n battletech --timeout=120s
   ```

5. Verify the API is healthy:
   ```
   curl -sf https://api.battledroids.ru/graphql -H 'Content-Type: application/json' \
     -d '{"query":"{ metadata { version } }"}' | python3 -m json.tool
   ```

## Service: roster

Run from the `../battletech-roster-builder/` repo root. Stop on any failure:

1. Push commits to origin:
   ```
   git push origin main
   ```

2. Build the amd64 Docker image and push to GHCR:
   ```
   docker buildx build --platform linux/amd64 \
     -t ghcr.io/graylikeme/battletech-roster:sha-$(git rev-parse --short HEAD) \
     -t ghcr.io/graylikeme/battletech-roster:latest \
     --push .
   ```

3. Restart the K8s deployment to pull the new image:
   ```
   KUBECONFIG=~/.kube/battletech.yaml kubectl rollout restart deployment/battletech-roster -n battletech
   ```

4. Wait for rollout to complete:
   ```
   KUBECONFIG=~/.kube/battletech.yaml kubectl rollout status deployment/battletech-roster -n battletech --timeout=120s
   ```

5. Verify the site is healthy:
   ```
   curl -sf -o /dev/null -w '%{http_code}' https://roster.battledroids.ru/
   ```

Report the result of each step. If the Docker build takes a while, run it in the background and check on it.
