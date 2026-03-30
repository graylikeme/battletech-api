#!/usr/bin/env bash
set -euo pipefail

# Uploads record-sheet assets to the S3 bucket under the roster/ prefix.
# Reads S3 credentials from Terraform outputs.
#
# Usage: ./scripts/upload-assets.sh <path-to-record-sheet/assets>
# Run from the infra/ directory.

ASSETS_DIR="${1:?Usage: $0 <path-to-record-sheet/assets>}"

if [ ! -d "$ASSETS_DIR/patterns" ]; then
  echo "Error: $ASSETS_DIR/patterns not found. Pass the record-sheet assets directory." >&2
  exit 1
fi

S3_HOSTNAME=$(terraform output -raw s3_bucket_hostname | sed 's|^https://||')
S3_ACCESS_KEY=$(terraform output -raw s3_access_key)
S3_SECRET_KEY=$(terraform output -raw s3_secret_key)
BUCKET_NAME=$(terraform output -raw s3_bucket_full_name)

export AWS_ACCESS_KEY_ID="$S3_ACCESS_KEY"
export AWS_SECRET_ACCESS_KEY="$S3_SECRET_KEY"

ENDPOINT="https://${S3_HOSTNAME}"

echo "Configuring CORS..."
aws s3api put-bucket-cors \
  --bucket "$BUCKET_NAME" \
  --endpoint-url "$ENDPOINT" \
  --cors-configuration '{
    "CORSRules": [
      {
        "AllowedOrigins": ["https://roster.battledroids.ru"],
        "AllowedMethods": ["GET", "HEAD"],
        "AllowedHeaders": ["*"],
        "MaxAgeSeconds": 86400
      }
    ]
  }'

echo "Uploading template images to roster/templates/..."
for f in RS_TW_BP.png RS_TW_QD.png Charts.png ChartsQD.png charts-minimal.png charts-minimalQD.png; do
  if [ -f "$ASSETS_DIR/$f" ]; then
    aws s3 cp "$ASSETS_DIR/$f" "s3://${BUCKET_NAME}/roster/templates/$f" \
      --endpoint-url "$ENDPOINT" \
      --content-type "image/png"
  fi
done

echo "Syncing patterns to roster/patterns/..."
aws s3 sync "$ASSETS_DIR/patterns/" "s3://${BUCKET_NAME}/roster/patterns/" \
  --endpoint-url "$ENDPOINT" \
  --content-type "image/png"

echo "Done. Files available at https://resources.battledroids.ru/roster/"
