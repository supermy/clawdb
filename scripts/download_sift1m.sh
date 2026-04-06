#!/bin/bash

set -e

DATA_DIR="$(cd "$(dirname "$0")/.." && pwd)/data"

SIFT_URLS=(
    "ftp://ftp.irisa.fr/local/texmex/corpus/sift1m.tar.gz"
    "https://www.dropbox.com/s/jvc5v0oos38j4aq/sift1m.tar.gz?dl=1"
)

echo "=== SIFT1M Dataset Download Script ==="
echo "Data directory: $DATA_DIR"

mkdir -p "$DATA_DIR"
cd "$DATA_DIR"

if [ -f "sift_base.fvecs" ] && [ -f "sift_query.fvecs" ] && [ -f "sift_groundtruth.ivecs" ]; then
    echo "SIFT1M dataset already exists. Skipping download."
    echo "Files:"
    ls -lh sift_*.fvecs sift_*.ivecs 2>/dev/null || true
    exit 0
fi

download_success=false
for url in "${SIFT_URLS[@]}"; do
    echo "Trying to download from: $url"
    echo "This may take a few minutes..."
    
    if command -v wget &> /dev/null; then
        if wget -O sift1m.tar.gz "$url" 2>/dev/null; then
            download_success=true
            break
        fi
    elif command -v curl &> /dev/null; then
        if curl -L -o sift1m.tar.gz "$url" 2>/dev/null; then
            download_success=true
            break
        fi
    fi
    echo "Failed to download from: $url"
    echo "Trying next mirror..."
done

if [ "$download_success" = false ]; then
    echo ""
    echo "Error: Failed to download SIFT1M dataset from all mirrors."
    echo ""
    echo "Please download manually from one of these sources:"
    echo "  - https://github.com/erikbern/ann-benchmarks#data-sets"
    echo "  - http://corpus-texmex.irisa.fr/"
    echo ""
    echo "Then extract the following files to the data/ directory:"
    echo "  - sift_base.fvecs (1,000,000 vectors, 128 dimensions)"
    echo "  - sift_query.fvecs (10,000 query vectors)"
    echo "  - sift_groundtruth.ivecs (ground truth results)"
    echo ""
    echo "Alternatively, you can generate synthetic test data:"
    echo "  cargo run --bin generate_test_data"
    exit 1
fi

echo "Extracting dataset..."
tar -xzf sift1m.tar.gz

echo "Moving files to data directory..."
if [ -d "sift1m" ]; then
    mv sift1m/*.fvecs . 2>/dev/null || true
    mv sift1m/*.ivecs . 2>/dev/null || true
    rm -rf sift1m
fi

rm -f sift1m.tar.gz

echo ""
echo "=== Download Complete ==="
echo "Files:"
ls -lh sift_*.fvecs sift_*.ivecs 2>/dev/null || true

echo ""
echo "Dataset Statistics:"
echo "- sift_base.fvecs: 1,000,000 vectors, 128 dimensions"
echo "- sift_query.fvecs: 10,000 query vectors, 128 dimensions"
echo "- sift_groundtruth.ivecs: 10,000 ground truth results"
